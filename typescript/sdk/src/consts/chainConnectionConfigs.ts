import { ethers } from 'ethers';

import { StaticCeloJsonRpcProvider } from '@hyperlane-xyz/celo-ethers-provider';

import { ChainMap, ChainName, IChainConnection } from '../types';
import { objMap } from '../utils/objects';

import { chainMetadata } from './chainMetadata';
// import { Chains, Mainnets, TestChains, Testnets } from './chains';
import { Chains } from './chains';

function testChainConnection() {
  return {
    provider: new ethers.providers.JsonRpcProvider(
      'http://localhost:8545',
      31337,
    ),
    confirmations: 1,
  };
}

// function khalaChainConnection() {
//   return {
//     provider: new ethers.providers.JsonRpcProvider(
//       'https://axon-node.info:8545',
//       100012,
//     ),
//     confirmations: 1,
//   };
// }

// function goerliChainConnection() {
//   return {
//     provider: new ethers.providers.JsonRpcProvider(
//       'https://goerli.infura.io/v3/a331eeeb1b1347a5a208925eda7167f6',
//       5,
//     ),
//     confirmations: 1,
//   };
// }

export const chainConnectionConfigs: ChainMap<ChainName, IChainConnection> =
  objMap(chainMetadata, (chainName, metadata) => {
    // if (TestChains.includes(chainName)) return testChainConnection();
    // if (Testnets.includes("goerli")) return goerliChainConnection();
    // if (Mainnets.includes("khala")) return khalaChainConnection();

    const providerClass =
      chainName === Chains.alfajores || chainName === Chains.celo
        ? StaticCeloJsonRpcProvider
        : ethers.providers.JsonRpcProvider;

    return {
      provider: new providerClass(metadata.publicRpcUrls[0].http, metadata.id),
      confirmations: metadata.blocks.confirmations,
      blockExplorerUrl: metadata.blockExplorers[0].url,
      blockExplorerApiUrl: metadata.blockExplorers[0].apiUrl,
    };
  });

export const testChainConnectionConfigs = {
  test1: testChainConnection(),
  test2: testChainConnection(),
  test3: testChainConnection(),
};
