import { InterchainQueryRouter__factory } from '@hyperlane-xyz/core';

import { proxiedFactories } from '../../router/types';

export const interchainQueryFactories = {
  interchainQueryRouter: new InterchainQueryRouter__factory(),
  ...proxiedFactories,
};

export type InterchainQueryFactories = typeof interchainQueryFactories;
