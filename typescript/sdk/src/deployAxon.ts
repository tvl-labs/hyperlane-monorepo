import { ChainMap } from '.';
// import '@nomiclabs/hardhat-waffle';
// import { ethers } from 'hardhat';
import { ethers } from 'ethers';

import { HyperlaneCore } from './core/HyperlaneCore';
import { HyperlaneCoreDeployer } from './deploy/core/HyperlaneCoreDeployer';
import { CoreConfig } from './deploy/core/types';
import { RouterConfig } from './deploy/router/types';
import { EnvironmentConfig } from './deploy/types';
import { getChainToOwnerMap, getKhalaMultiProvider } from './deploy/utils';
import { MultiProvider } from './providers/MultiProvider';
import { RouterContracts } from './router';
// Create Deploy contracts to Godwoken, Axon and Goerli (Maybe use Sepolia?)
// Create a test environment for the contracts
// Deploy Application?
import {
  EnvSubsetApp,
  EnvSubsetChecker,
  EnvSubsetDeployer,
  KhalaSubsetChains, // SubsetChains,
  // fullEnvConfigs,
  // fullEnvTestConfigs,
  subsetKhalaConfigs, // subsetTestConfigs,
} from './test/envSubsetDeployer/app';
import { ChainName, KhalaChainNames } from './types';

require('dotenv').config();

const provider = new ethers.providers.JsonRpcProvider(
  'https://www.axon-node.info/',
);

process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

let ismOwnerAddress = ethers.utils.getAddress(
  '0xe7d5869FE1955F2500987B9eCCFF0a9452c164cf',
);
let validatorAddress = [
  ethers.utils.getAddress('0xe7d5869FE1955F2500987B9eCCFF0a9452c164cf'),
];

let multisigIsmConfig: CoreConfig = {
  owner: ismOwnerAddress,
  multisigIsm: {
    validators: validatorAddress,
    threshold: 1,
  },
};

const configs = {
  khala: multisigIsmConfig,
  goerli: multisigIsmConfig,
} as ChainMap<ChainName, CoreConfig>;

// import { ChainMap } from '.';
async function main() {
  const env = await initEnv(subsetKhalaConfigs);
  let multiProvider: MultiProvider<KhalaSubsetChains>;
  let config: ChainMap<KhalaSubsetChains, RouterConfig>;
  let deployer: EnvSubsetDeployer<KhalaSubsetChains>;
  let contracts: Record<KhalaSubsetChains, RouterContracts>;
  let app: EnvSubsetApp<KhalaSubsetChains>;

  config = {
    khala: env.config.khala,
    goerli: env.config.goerli,
  };

  multiProvider = env.multiProvider;
  deployer = env.deployer;
  contracts = await deployer.deploy();
  app = new EnvSubsetApp(contracts, multiProvider);
  const checker = new EnvSubsetChecker(multiProvider, app, config);
  console.log(`checker: ${checker}`);
  // app = new EnvSubsetApp(fullEnvTestConfigs);
  // await app.init();
}

async function initEnv<Chain extends KhalaChainNames>(
  environmentConfig: EnvironmentConfig<Chain>,
) {
  // const [signer] = await ethers.getSigners();
  // const signer = new ethers.Wallet(process.env.PRIVATE_KEY, provider);
  const signer = new ethers.Wallet(
    '3f979f04df632edc0e28a478b9adc34c9262a1a52c7597664e442b9c0093b8f2',
    provider,
  );

  const multiProvider = getKhalaMultiProvider(signer, environmentConfig);
  console.log(`multiProvider: ${JSON.stringify(multiProvider)}`);

  const coreDeployer = new HyperlaneCoreDeployer(multiProvider, configs);

  // console.log(`coreDeployer: ${JSON.stringify(coreDeployer)}`);

  const coreContractsMaps = await coreDeployer.deploy();
  console.log(`coreContractsMaps: ${JSON.stringify(coreContractsMaps)}`);
  const core = new HyperlaneCore(coreContractsMaps, multiProvider);
  const config = core.extendWithConnectionClientConfig(
    // getChainToOwnerMap(subsetKhalaConfigs, signer.address),
    getChainToOwnerMap(subsetKhalaConfigs, ismOwnerAddress),
  );
  const deployer = new EnvSubsetDeployer(multiProvider, config, core);
  return { multiProvider, config, deployer };
}

main()
  .then(() => console.info('Deploy complete'))
  .catch(console.error);
