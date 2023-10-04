import { ChainMap, ChainMetadata, chainMetadata } from '@hyperlane-xyz/sdk';

import { AgentChainNames, Role } from '../../../src/roles';

// Blessed
export const ethereumTestnetConfigs: ChainMap<ChainMetadata> = {
  alfajores: chainMetadata.alfajores,
  fuji: chainMetadata.fuji,
  mumbai: {
    ...chainMetadata.mumbai,
    transactionOverrides: {
      maxFeePerGas: 150 * 10 ** 9, // 70 gwei
      maxPriorityFeePerGas: 40 * 10 ** 9, // 40 gwei
    },
  },
  bsctestnet: chainMetadata.bsctestnet,
  goerli: chainMetadata.goerli,
  sepolia: chainMetadata.sepolia,
  moonbasealpha: chainMetadata.moonbasealpha,
  optimismgoerli: chainMetadata.optimismgoerli,
  arbitrumgoerli: chainMetadata.arbitrumgoerli,
};

// Blessed non-Ethereum chains.
export const nonEthereumTestnetConfigs: ChainMap<ChainMetadata> = {
  solanadevnet: chainMetadata.solanadevnet,
};

export const testnetConfigs: ChainMap<ChainMetadata> = {
  ...ethereumTestnetConfigs,
  ...nonEthereumTestnetConfigs,
};

export type TestnetChains = keyof typeof testnetConfigs;
export const supportedChainNames = Object.keys(
  testnetConfigs,
) as TestnetChains[];

export const ethereumChainNames = Object.keys(
  ethereumTestnetConfigs,
) as TestnetChains[];

export const environment = 'testnet3';

const validatorChainNames = [
  ...supportedChainNames,
  chainMetadata.proteustestnet.name,
];

const relayerChainNames = validatorChainNames;

export const agentChainNames: AgentChainNames = {
  [Role.Validator]: validatorChainNames,
  [Role.Relayer]: relayerChainNames,
  [Role.Scraper]: ethereumChainNames,
};
