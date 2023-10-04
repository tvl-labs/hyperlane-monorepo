import debug from 'debug';

import { isObject } from '@hyperlane-xyz/utils';

import { HyperlaneContracts, HyperlaneContractsMap } from '../contracts/types';
import { HyperlaneDeployer } from '../deploy/HyperlaneDeployer';
import { MultiProvider } from '../providers/MultiProvider';
import { ChainMap, ChainName } from '../types';

import { IsmFactoryFactories, ismFactoryFactories } from './contracts';

export class HyperlaneIsmFactoryDeployer extends HyperlaneDeployer<
  boolean,
  IsmFactoryFactories
> {
  constructor(multiProvider: MultiProvider) {
    super(multiProvider, ismFactoryFactories, {
      logger: debug('hyperlane:IsmFactoryDeployer'),
    });
  }

  async deploy(
    config: ChainName[] | ChainMap<boolean>,
  ): Promise<HyperlaneContractsMap<IsmFactoryFactories>> {
    if (isObject(config)) {
      return super.deploy(config as ChainMap<boolean>);
    } else {
      return super.deploy(
        Object.fromEntries((config as ChainName[]).map((c) => [c, true])),
      );
    }
  }

  async deployContracts(
    chain: ChainName,
  ): Promise<HyperlaneContracts<IsmFactoryFactories>> {
    const merkleRootMultisigIsmFactory = await this.deployContract(
      chain,
      'merkleRootMultisigIsmFactory',
      [],
    );
    const messageIdMultisigIsmFactory = await this.deployContract(
      chain,
      'messageIdMultisigIsmFactory',
      [],
    );
    const aggregationIsmFactory = await this.deployContract(
      chain,
      'aggregationIsmFactory',
      [],
    );
    const routingIsmFactory = await this.deployContract(
      chain,
      'routingIsmFactory',
      [],
    );
    return {
      merkleRootMultisigIsmFactory,
      messageIdMultisigIsmFactory,
      aggregationIsmFactory,
      routingIsmFactory,
    };
  }
}
