import { ChainName } from '../types';

import { HyperlaneRouterChecker } from './HyperlaneRouterChecker';
import { RouterApp } from './RouterApps';
import { ProxiedFactories, ProxiedRouterConfig } from './types';

export abstract class ProxiedRouterChecker<
  Factories extends ProxiedFactories,
  App extends RouterApp<Factories>,
  Config extends ProxiedRouterConfig,
> extends HyperlaneRouterChecker<Factories, App, Config> {
  async checkOwnership(chain: ChainName): Promise<void> {
    const config = this.configMap[chain];
    let ownableOverrides = {};
    if (config.timelock) {
      ownableOverrides = {
        proxyAdmin: this.app.getAddresses(chain).timelockController,
      };
    }

    return super.checkOwnership(chain, config.owner, ownableOverrides);
  }

  async checkChain(chain: ChainName): Promise<void> {
    await super.checkHyperlaneConnectionClient(chain);
    await super.checkEnrolledRouters(chain);
    await this.checkProxiedContracts(chain);
    await this.checkOwnership(chain);
  }
}
