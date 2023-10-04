import { Debugger, debug } from 'debug';
import { Contract, ethers } from 'ethers';

import {
  HyperlaneConnectionClient,
  Mailbox,
  Mailbox__factory,
  Ownable,
  ProxyAdmin,
  ProxyAdmin__factory,
  TimelockController,
  TimelockController__factory,
  TransparentUpgradeableProxy,
  TransparentUpgradeableProxy__factory,
} from '@hyperlane-xyz/core';
import { Address, eqAddress, runWithTimeout } from '@hyperlane-xyz/utils';

import {
  HyperlaneAddressesMap,
  HyperlaneContracts,
  HyperlaneContractsMap,
  HyperlaneFactories,
} from '../contracts/types';
import {
  HyperlaneIsmFactory,
  moduleMatchesConfig,
} from '../ism/HyperlaneIsmFactory';
import { MultiProvider } from '../providers/MultiProvider';
import { ConnectionClientConfig } from '../router/types';
import { ChainMap, ChainName } from '../types';

import { UpgradeConfig, proxyAdmin } from './proxy';
import { ContractVerificationInput } from './verify/types';
import { getContractVerificationInput } from './verify/utils';

export interface DeployerOptions {
  logger?: Debugger;
  chainTimeoutMs?: number;
  ismFactory?: HyperlaneIsmFactory;
}

export abstract class HyperlaneDeployer<
  Config,
  Factories extends HyperlaneFactories,
> {
  public verificationInputs: ChainMap<ContractVerificationInput[]> = {};
  public cachedAddresses: HyperlaneAddressesMap<any> = {};
  public deployedContracts: HyperlaneContractsMap<Factories> = {};
  public startingBlockNumbers: ChainMap<number | undefined> = {};

  protected logger: Debugger;
  protected chainTimeoutMs: number;

  constructor(
    protected readonly multiProvider: MultiProvider,
    protected readonly factories: Factories,
    protected readonly options?: DeployerOptions,
  ) {
    this.logger = options?.logger ?? debug('hyperlane:deployer');
    this.chainTimeoutMs = options?.chainTimeoutMs ?? 5 * 60 * 1000; // 5 minute timeout per chain
  }

  cacheAddressesMap(addressesMap: HyperlaneAddressesMap<any>): void {
    this.cachedAddresses = addressesMap;
  }

  async checkConfig(_: ChainMap<Config>): Promise<void> {
    return;
  }

  abstract deployContracts(
    chain: ChainName,
    config: Config,
  ): Promise<HyperlaneContracts<Factories>>;

  async deploy(
    configMap: ChainMap<Config>,
  ): Promise<HyperlaneContractsMap<Factories>> {
    const configChains = Object.keys(configMap);
    const targetChains = this.multiProvider.intersect(
      configChains,
      true,
    ).intersection;

    this.logger(`Start deploy to ${targetChains}`);
    for (const chain of targetChains) {
      const signerUrl = await this.multiProvider.tryGetExplorerAddressUrl(
        chain,
      );
      const signerAddress = await this.multiProvider.getSignerAddress(chain);
      const fromString = signerUrl || signerAddress;
      this.logger(`Deploying to ${chain} from ${fromString}`);
      this.startingBlockNumbers[chain] = await this.multiProvider
        .getProvider(chain)
        .getBlockNumber();
      await runWithTimeout(this.chainTimeoutMs, async () => {
        this.deployedContracts[chain] = await this.deployContracts(
          chain,
          configMap[chain],
        );
      });
    }
    return this.deployedContracts;
  }

  protected async runIf<T>(
    chain: ChainName,
    address: string,
    fn: () => Promise<T>,
    label = 'address',
  ): Promise<T | undefined> {
    const signer = await this.multiProvider.getSignerAddress(chain);
    if (eqAddress(address, signer)) {
      return fn();
    } else {
      this.logger(`Signer (${signer}) does not match ${label} (${address})`);
    }
    return undefined;
  }

  protected async runIfOwner<T>(
    chain: ChainName,
    ownable: Ownable,
    fn: () => Promise<T>,
  ): Promise<T | undefined> {
    return this.runIf(chain, await ownable.callStatic.owner(), fn, 'owner');
  }

  protected async runIfAdmin<T>(
    chain: ChainName,
    proxy: Contract,
    signerAdminFn: () => Promise<T>,
    proxyAdminOwnerFn: (proxyAdmin: ProxyAdmin) => Promise<T>,
  ): Promise<T | undefined> {
    const admin = await proxyAdmin(
      this.multiProvider.getProvider(chain),
      proxy.address,
    );
    const code = await this.multiProvider.getProvider(chain).getCode(admin);
    // if admin is a ProxyAdmin, run the proxyAdminOwnerFn (if deployer is owner)
    if (code !== '0x') {
      this.logger(`Admin is a ProxyAdmin (${admin})`);
      const proxyAdmin = ProxyAdmin__factory.connect(admin, proxy.signer);
      return this.runIfOwner(chain, proxyAdmin, () =>
        proxyAdminOwnerFn(proxyAdmin),
      );
    } else {
      this.logger(`Admin is an EOA (${admin})`);
      // if admin is an EOA, run the signerAdminFn (if deployer is admin)
      return this.runIf(chain, admin, () => signerAdminFn(), 'admin');
    }
  }

  protected async initConnectionClient(
    local: ChainName,
    connectionClient: HyperlaneConnectionClient,
    config: ConnectionClientConfig,
  ): Promise<void> {
    this.logger(
      `Initializing connection client (if not already) on ${local}...`,
    );
    await this.runIfOwner(local, connectionClient, async () => {
      const txOverrides = this.multiProvider.getTransactionOverrides(local);
      // set mailbox if not already set (and configured)
      if (config.mailbox !== (await connectionClient.mailbox())) {
        this.logger(`Set mailbox on (${local})`);
        await this.multiProvider.handleTx(
          local,
          connectionClient.setMailbox(config.mailbox, txOverrides),
        );
      }

      // set interchain gas paymaster if not already set (and configured)
      if (
        config.interchainGasPaymaster !==
        (await connectionClient.interchainGasPaymaster())
      ) {
        this.logger(`Set interchain gas paymaster on ${local}`);
        await this.multiProvider.handleTx(
          local,
          connectionClient.setInterchainGasPaymaster(
            config.interchainGasPaymaster,
            txOverrides,
          ),
        );
      }

      let currentIsm = await connectionClient.interchainSecurityModule();
      // in case the above returns zero address, fetch the defaultISM from the mailbox
      if (currentIsm === ethers.constants.AddressZero) {
        const mailbox: Mailbox = Mailbox__factory.connect(
          config.mailbox,
          connectionClient.signer,
        );
        currentIsm = await mailbox.defaultIsm();
      }

      if (config.interchainSecurityModule) {
        // set interchain security module if not already set (and configured)
        let configuredIsm: string;
        if (typeof config.interchainSecurityModule === 'string') {
          configuredIsm = config.interchainSecurityModule;
        } else if (this.options?.ismFactory) {
          const matches = await moduleMatchesConfig(
            local,
            currentIsm,
            config.interchainSecurityModule,
            this.multiProvider,
            this.options.ismFactory.chainMap[local],
          );
          if (matches) {
            // when the ISM recursively matches the IsmConfig, we don't need to deploy a new ISM
            this.logger(
              `ISM matches config for chain ${local}, skipping deploy`,
            );
            return;
          }
          const ism = await this.options.ismFactory.deploy(
            local,
            config.interchainSecurityModule,
          );
          configuredIsm = ism.address;
        } else {
          throw new Error('No ISM factory provided');
        }

        if (!eqAddress(currentIsm, configuredIsm)) {
          this.logger(
            `Set interchain security module on ${local} at ${configuredIsm}`,
          );

          await this.multiProvider.handleTx(
            local,
            connectionClient.setInterchainSecurityModule(
              configuredIsm,
              txOverrides,
            ),
          );
        }
      }
    });
    this.logger(`Connection client on ${local} initialized...`);
  }

  protected async deployContractFromFactory<F extends ethers.ContractFactory>(
    chain: ChainName,
    factory: F,
    contractName: string,
    constructorArgs: Parameters<F['deploy']>,
    initializeArgs?: Parameters<Awaited<ReturnType<F['deploy']>>['initialize']>,
  ): Promise<ReturnType<F['deploy']>> {
    const cachedContract = this.readCache(chain, factory, contractName);
    if (cachedContract) {
      return cachedContract;
    }

    const signer = this.multiProvider.getSigner(chain);
    const overrides = this.multiProvider.getTransactionOverrides(chain);

    this.logger(`Deploy ${contractName} on ${chain}`);
    const contract = await (factory
      .connect(signer)
      .deploy(...constructorArgs, overrides) as ReturnType<F['deploy']>);

    await this.multiProvider.handleTx(chain, contract.deployTransaction);

    if (initializeArgs) {
      this.logger(`Initialize ${contractName} on ${chain}`);
      const initTx = await contract.initialize(...initializeArgs, overrides);
      await this.multiProvider.handleTx(chain, initTx);
    }

    const verificationInput = getContractVerificationInput(
      contractName,
      contract,
      factory.bytecode,
    );
    this.verificationInputs[chain] = this.verificationInputs[chain] || [];
    this.verificationInputs[chain].push(verificationInput);

    return contract;
  }

  async deployContract<K extends keyof Factories>(
    chain: ChainName,
    contractName: K,
    constructorArgs: Parameters<Factories[K]['deploy']>,
    initializeArgs?: Parameters<
      Awaited<ReturnType<Factories[K]['deploy']>>['initialize']
    >,
  ): Promise<HyperlaneContracts<Factories>[K]> {
    const contract = (await this.deployContractFromFactory(
      chain,
      this.factories[contractName],
      contractName.toString(),
      constructorArgs,
      initializeArgs,
    )) as HyperlaneContracts<Factories>[K];
    this.writeCache(chain, contractName, contract.address);
    return contract;
  }

  protected async changeAdmin(
    chain: ChainName,
    proxy: TransparentUpgradeableProxy,
    admin: string,
  ): Promise<void> {
    const actualAdmin = await proxyAdmin(
      this.multiProvider.getProvider(chain),
      proxy.address,
    );
    if (eqAddress(admin, actualAdmin)) {
      this.logger(`Admin set correctly, skipping admin change`);
      return;
    }

    const txOverrides = this.multiProvider.getTransactionOverrides(chain);
    this.logger(`Changing proxy admin`);
    await this.runIfAdmin(
      chain,
      proxy,
      () =>
        this.multiProvider.handleTx(
          chain,
          proxy.changeAdmin(admin, txOverrides),
        ),
      (proxyAdmin) =>
        this.multiProvider.handleTx(
          chain,
          proxyAdmin.changeProxyAdmin(proxy.address, admin, txOverrides),
        ),
    );
  }

  protected async upgradeAndInitialize<C extends ethers.Contract>(
    chain: ChainName,
    proxy: TransparentUpgradeableProxy,
    implementation: C,
    initializeArgs: Parameters<C['initialize']>,
  ): Promise<void> {
    const current = await proxy.callStatic.implementation();
    if (eqAddress(implementation.address, current)) {
      this.logger(`Implementation set correctly, skipping upgrade`);
      return;
    }

    this.logger(`Upgrading and initializing implementation`);
    const initData = implementation.interface.encodeFunctionData(
      'initialize',
      initializeArgs,
    );
    const overrides = this.multiProvider.getTransactionOverrides(chain);
    await this.runIfAdmin(
      chain,
      proxy,
      () =>
        this.multiProvider.handleTx(
          chain,
          proxy.upgradeToAndCall(implementation.address, initData, overrides),
        ),
      (proxyAdmin: ProxyAdmin) =>
        this.multiProvider.handleTx(
          chain,
          proxyAdmin.upgradeAndCall(
            proxy.address,
            implementation.address,
            initData,
            overrides,
          ),
        ),
    );
  }

  protected async deployProxy<C extends ethers.Contract>(
    chain: ChainName,
    implementation: C,
    proxyAdmin: string,
    initializeArgs?: Parameters<C['initialize']>,
  ): Promise<C> {
    const initData = initializeArgs
      ? implementation.interface.encodeFunctionData(
          'initialize',
          initializeArgs,
        )
      : '0x';

    this.logger(`Deploying transparent upgradable proxy`);
    const constructorArgs: [string, string, string] = [
      implementation.address,
      proxyAdmin,
      initData,
    ];
    const proxy = await this.deployContractFromFactory(
      chain,
      new TransparentUpgradeableProxy__factory(),
      'TransparentUpgradeableProxy',
      constructorArgs,
    );

    return implementation.attach(proxy.address) as C;
  }

  async deployTimelock(
    chain: ChainName,
    timelockConfig: UpgradeConfig['timelock'],
  ): Promise<TimelockController> {
    const timelock = await this.deployContractFromFactory(
      chain,
      new TimelockController__factory(),
      'timelockController',
      // delay, [proposers], [executors], admin
      [
        timelockConfig.delay,
        [timelockConfig.roles.proposer],
        [timelockConfig.roles.executor],
        ethers.constants.AddressZero,
      ],
    );
    return timelock;
  }

  protected writeCache<K extends keyof Factories>(
    chain: ChainName,
    contractName: K,
    address: Address,
  ): void {
    if (!this.cachedAddresses[chain]) {
      this.cachedAddresses[chain] = {};
    }
    this.cachedAddresses[chain][contractName] = address;
  }

  protected readCache<F extends ethers.ContractFactory>(
    chain: ChainName,
    factory: F,
    contractName: string,
  ): Awaited<ReturnType<F['deploy']>> | undefined {
    const cachedAddress = this.cachedAddresses[chain]?.[contractName];
    const hit =
      !!cachedAddress && cachedAddress !== ethers.constants.AddressZero;
    const contractAddress = hit ? cachedAddress : ethers.constants.AddressZero;
    const contract = factory
      .attach(contractAddress)
      .connect(this.multiProvider.getSignerOrProvider(chain)) as Awaited<
      ReturnType<F['deploy']>
    >;
    if (hit) {
      this.logger(
        `Recovered ${contractName.toString()} on ${chain} ${cachedAddress}`,
      );
      return contract;
    }
    return undefined;
  }

  /**
   * Deploys the Implementation and Proxy for a given contract
   *
   */
  async deployProxiedContract<K extends keyof Factories>(
    chain: ChainName,
    contractName: K,
    proxyAdmin: string,
    constructorArgs: Parameters<Factories[K]['deploy']>,
    initializeArgs?: Parameters<HyperlaneContracts<Factories>[K]['initialize']>,
  ): Promise<HyperlaneContracts<Factories>[K]> {
    const cachedContract = this.readCache(
      chain,
      this.factories[contractName],
      contractName.toString(),
    );
    if (cachedContract) {
      return cachedContract;
    }

    // Try to initialize the implementation even though it may not be necessary
    const implementation = await this.deployContract(
      chain,
      contractName,
      constructorArgs,
      initializeArgs,
    );

    // Initialize the proxy the same way
    const contract = await this.deployProxy(
      chain,
      implementation,
      proxyAdmin,
      initializeArgs,
    );
    this.writeCache(chain, contractName, contract.address);
    return contract;
  }

  mergeWithExistingVerificationInputs(
    existingInputsMap: ChainMap<ContractVerificationInput[]>,
  ): ChainMap<ContractVerificationInput[]> {
    const allChains = new Set<ChainName>();
    Object.keys(existingInputsMap).forEach((_) => allChains.add(_));
    Object.keys(this.verificationInputs).forEach((_) => allChains.add(_));

    const ret: ChainMap<ContractVerificationInput[]> = {};
    for (const chain of allChains) {
      const existingInputs = existingInputsMap[chain] || [];
      const newInputs = this.verificationInputs[chain] || [];
      ret[chain] = [...existingInputs, ...newInputs];
    }
    return ret;
  }

  protected async transferOwnershipOfContracts(
    chain: ChainName,
    owner: Address,
    ownables: { [key: string]: Ownable },
  ): Promise<ethers.ContractReceipt[]> {
    const receipts: ethers.ContractReceipt[] = [];
    for (const contractName of Object.keys(ownables)) {
      const ownable = ownables[contractName];
      const currentOwner = await ownable.owner();
      if (!eqAddress(currentOwner, owner)) {
        this.logger(
          `Transferring ownership of ${contractName} to ${owner} on ${chain}`,
        );
        const receipt = await this.runIfOwner(chain, ownable, () =>
          this.multiProvider.handleTx(
            chain,
            ownable.transferOwnership(
              owner,
              this.multiProvider.getTransactionOverrides(chain),
            ),
          ),
        );
        if (receipt) receipts.push(receipt);
      }
    }

    return receipts.filter((x) => !!x) as ethers.ContractReceipt[];
  }
}
