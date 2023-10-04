import {
  HyperlaneConnectionClient,
  ProxyAdmin__factory,
  Router,
  TimelockController__factory,
} from '@hyperlane-xyz/core';
import type { Address } from '@hyperlane-xyz/utils';

import { HyperlaneFactories } from '../contracts/types';
import { UpgradeConfig } from '../deploy/proxy';
import { CheckerViolation } from '../deploy/types';
import { IsmConfig } from '../ism/types';

export type RouterAddress = {
  router: Address;
};

export type OwnableConfig = {
  owner: Address;
};

export type ForeignDeploymentConfig = {
  foreignDeployment?: Address;
};

export type RouterConfig = ConnectionClientConfig &
  OwnableConfig &
  ForeignDeploymentConfig;

export type ProxiedRouterConfig = RouterConfig & Partial<UpgradeConfig>;

export type GasConfig = {
  gas: number;
};

export type GasRouterConfig = RouterConfig & GasConfig;

export type ProxiedFactories = HyperlaneFactories & {
  proxyAdmin: ProxyAdmin__factory;
  timelockController: TimelockController__factory;
};

export const proxiedFactories: ProxiedFactories = {
  proxyAdmin: new ProxyAdmin__factory(),
  timelockController: new TimelockController__factory(),
};

export type ConnectionClientConfig = {
  mailbox: Address;
  interchainGasPaymaster: Address;
  interchainSecurityModule?: Address | IsmConfig;
};

export enum ConnectionClientViolationType {
  InterchainSecurityModule = 'ConnectionClientIsm',
  Mailbox = 'ConnectionClientMailbox',
  InterchainGasPaymaster = 'ConnectionClientIgp',
}

export interface ConnectionClientViolation extends CheckerViolation {
  type: ConnectionClientViolationType;
  contract: HyperlaneConnectionClient;
  actual: string;
  expected: string;
  description?: string;
}

export enum RouterViolationType {
  EnrolledRouter = 'EnrolledRouter',
}

export interface RouterViolation extends CheckerViolation {
  type: RouterViolationType.EnrolledRouter;
  remoteChain: string;
  contract: Router;
  actual: string;
  expected: string;
}
