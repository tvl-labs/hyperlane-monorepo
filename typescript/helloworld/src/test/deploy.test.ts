import '@nomiclabs/hardhat-waffle';
import { ethers } from 'hardhat';

import {
  ChainMap,
  HyperlaneContractsMap,
  MultiProvider,
  TestCoreApp,
  TestCoreDeployer,
  deployTestIgpsAndGetRouterConfig,
} from '@hyperlane-xyz/sdk';

import { HelloWorldApp } from '../app/app';
import { HelloWorldFactories } from '../app/contracts';
import { HelloWorldChecker } from '../deploy/check';
import { HelloWorldConfig } from '../deploy/config';
import { HelloWorldDeployer } from '../deploy/deploy';

describe('deploy', async () => {
  let multiProvider: MultiProvider;
  let core: TestCoreApp;
  let config: ChainMap<HelloWorldConfig>;
  let deployer: HelloWorldDeployer;
  let contracts: HyperlaneContractsMap<HelloWorldFactories>;
  let app: HelloWorldApp;

  before(async () => {
    const [signer] = await ethers.getSigners();
    multiProvider = MultiProvider.createTestMultiProvider({ signer });

    const coreDeployer = new TestCoreDeployer(multiProvider);
    core = await coreDeployer.deployApp();
    config = await deployTestIgpsAndGetRouterConfig(
      multiProvider,
      signer.address,
      core.contractsMap,
    );
    deployer = new HelloWorldDeployer(multiProvider);
  });

  it('deploys', async () => {
    contracts = await deployer.deploy(config);
  });

  it('builds app', async () => {
    contracts = await deployer.deploy(config);
    app = new HelloWorldApp(core, contracts, multiProvider);
  });

  it('checks', async () => {
    const checker = new HelloWorldChecker(multiProvider, app, config);
    await checker.check();
    checker.expectEmpty();
  });
});
