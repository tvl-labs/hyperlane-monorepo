import { InterchainQueryRouter } from '@hyperlane-xyz/core';

import {
  HyperlaneEnvironment,
  hyperlaneEnvironments,
} from '../../consts/environments';
import { appFromAddressesMapHelper } from '../../contracts/contracts';
import {
  HyperlaneAddressesMap,
  HyperlaneContracts,
} from '../../contracts/types';
import { MultiProvider } from '../../providers/MultiProvider';
import { RouterApp } from '../../router/RouterApps';

import {
  InterchainQueryFactories,
  interchainQueryFactories,
} from './contracts';

export class InterchainQuery extends RouterApp<InterchainQueryFactories> {
  router(
    contracts: HyperlaneContracts<InterchainQueryFactories>,
  ): InterchainQueryRouter {
    return contracts.interchainQueryRouter;
  }

  static fromEnvironment<Env extends HyperlaneEnvironment>(
    env: Env,
    multiProvider: MultiProvider,
  ): InterchainQuery {
    const envAddresses = hyperlaneEnvironments[env];
    if (!envAddresses) {
      throw new Error(`No addresses found for ${env}`);
    }
    return InterchainQuery.fromAddressesMap(envAddresses, multiProvider);
  }

  static fromAddressesMap(
    addressesMap: HyperlaneAddressesMap<any>,
    multiProvider: MultiProvider,
  ): InterchainQuery {
    const helper = appFromAddressesMapHelper(
      addressesMap,
      interchainQueryFactories,
      multiProvider,
    );
    return new InterchainQuery(helper.contractsMap, helper.multiProvider);
  }
}
