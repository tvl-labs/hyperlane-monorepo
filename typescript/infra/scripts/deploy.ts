import path from 'path';

import { HelloWorldDeployer } from '@hyperlane-xyz/helloworld';
import {
  ChainMap,
  HyperlaneCoreDeployer,
  HyperlaneDeployer,
  HyperlaneHookDeployer,
  HyperlaneIgp,
  HyperlaneIgpDeployer,
  HyperlaneIsmFactory,
  HyperlaneIsmFactoryDeployer,
  InterchainAccountDeployer,
  InterchainQueryDeployer,
  LiquidityLayerDeployer,
} from '@hyperlane-xyz/sdk';
import { objMap } from '@hyperlane-xyz/utils';

import { Contexts } from '../config/contexts';
import { deployEnvToSdkEnv } from '../src/config/environment';
import { deployWithArtifacts } from '../src/deployment/deploy';
import { TestQuerySenderDeployer } from '../src/deployment/testcontracts/testquerysender';
import { TestRecipientDeployer } from '../src/deployment/testcontracts/testrecipient';
import { impersonateAccount, useLocalProvider } from '../src/utils/fork';
import { readJSON } from '../src/utils/utils';

import {
  Modules,
  SDK_MODULES,
  getArgs,
  getContractAddressesSdkFilepath,
  getEnvironmentConfig,
  getEnvironmentDirectory,
  getModuleDirectory,
  getProxiedRouterConfig,
  getRouterConfig,
  withContext,
  withModuleAndFork,
} from './utils';

async function main() {
  const {
    context = Contexts.Hyperlane,
    module,
    fork,
    environment,
  } = await withContext(withModuleAndFork(getArgs())).argv;
  const envConfig = getEnvironmentConfig(environment);
  const multiProvider = await envConfig.getMultiProvider();

  if (fork) {
    await useLocalProvider(multiProvider, fork);

    // TODO: make this more generic
    const deployerAddress =
      environment === 'testnet3'
        ? '0xfaD1C94469700833717Fa8a3017278BC1cA8031C'
        : '0xa7ECcdb9Be08178f896c26b7BbD8C3D4E844d9Ba';

    const signer = await impersonateAccount(deployerAddress);
    multiProvider.setSharedSigner(signer);
  }

  let config: ChainMap<unknown> = {};
  let deployer: HyperlaneDeployer<any, any>;
  if (module === Modules.ISM_FACTORY) {
    config = objMap(envConfig.core, (_chain) => true);
    deployer = new HyperlaneIsmFactoryDeployer(multiProvider);
  } else if (module === Modules.CORE) {
    config = envConfig.core;
    const ismFactory = HyperlaneIsmFactory.fromEnvironment(
      deployEnvToSdkEnv[environment],
      multiProvider,
    );
    deployer = new HyperlaneCoreDeployer(multiProvider, ismFactory);
  } else if (module === Modules.HOOK) {
    config = envConfig.hooks;
    deployer = new HyperlaneHookDeployer(multiProvider);
  } else if (module === Modules.INTERCHAIN_GAS_PAYMASTER) {
    config = envConfig.igp;
    deployer = new HyperlaneIgpDeployer(multiProvider);
  } else if (module === Modules.INTERCHAIN_ACCOUNTS) {
    config = await getProxiedRouterConfig(environment, multiProvider);
    deployer = new InterchainAccountDeployer(multiProvider);
  } else if (module === Modules.INTERCHAIN_QUERY_SYSTEM) {
    config = await getProxiedRouterConfig(environment, multiProvider);
    deployer = new InterchainQueryDeployer(multiProvider);
  } else if (module === Modules.LIQUIDITY_LAYER) {
    const routerConfig = await getProxiedRouterConfig(
      environment,
      multiProvider,
    );
    if (!envConfig.liquidityLayerConfig) {
      throw new Error(`No liquidity layer config for ${environment}`);
    }
    config = objMap(
      envConfig.liquidityLayerConfig.bridgeAdapters,
      (chain, conf) => ({
        ...conf,
        ...routerConfig[chain],
      }),
    );
    deployer = new LiquidityLayerDeployer(multiProvider);
  } else if (module === Modules.TEST_RECIPIENT) {
    deployer = new TestRecipientDeployer(multiProvider);
  } else if (module === Modules.TEST_QUERY_SENDER) {
    // TODO: make this more generic
    const igp = HyperlaneIgp.fromEnvironment(
      deployEnvToSdkEnv[environment],
      multiProvider,
    );
    // Get query router addresses
    const queryRouterDir = path.join(
      getEnvironmentDirectory(environment),
      'middleware/queries',
    );
    config = objMap(readJSON(queryRouterDir, 'addresses.json'), (_c, conf) => ({
      queryRouterAddress: conf.router,
    }));
    deployer = new TestQuerySenderDeployer(multiProvider, igp);
  } else if (module === Modules.HELLO_WORLD) {
    config = await getRouterConfig(
      environment,
      multiProvider,
      true, // use deployer as owner
    );
    const ismFactory = HyperlaneIsmFactory.fromEnvironment(
      deployEnvToSdkEnv[environment],
      multiProvider,
    );
    deployer = new HelloWorldDeployer(multiProvider, ismFactory);
  } else {
    console.log(`Skipping ${module}, deployer unimplemented`);
    return;
  }

  const modulePath = getModuleDirectory(environment, module, context);

  console.log(`Deploying to ${modulePath}`);

  const addresses = SDK_MODULES.includes(module)
    ? path.join(
        getContractAddressesSdkFilepath(),
        `${deployEnvToSdkEnv[environment]}.json`,
      )
    : path.join(modulePath, 'addresses.json');

  const verification = path.join(modulePath, 'verification.json');

  const cache = {
    addresses,
    verification,
    read: environment !== 'test',
    write: true,
  };
  // Don't write agent config in fork tests
  const agentConfig =
    ['core', 'igp'].includes(module) && !fork
      ? {
          addresses,
          environment,
          multiProvider,
        }
      : undefined;

  await deployWithArtifacts(config, deployer, cache, fork, agentConfig);
}

main()
  .then()
  .catch((e) => {
    console.error(e);
    process.exit(1);
  });
