import { chainConnectionConfigs } from '../../.../../../../sdk';

export const khalaConfigs = {
  // bsc: {
  //   ...chainConnectionConfigs.bsc,
  //   overrides: {
  //     gasPrice: 7 * 10 ** 9, // 7 gwei
  //   },
  // },
  // avalanche: chainConnectionConfigs.avalanche,
  // polygon: {
  //   ...chainConnectionConfigs.polygon,
  //   confirmations: 3,
  //   overrides: {
  //     maxFeePerGas: 500 * 10 ** 9, // 500 gwei
  //     maxPriorityFeePerGas: 100 * 10 ** 9, // 100 gwei
  //     // gasPrice: 50 * 10 ** 9, // 50 gwei
  //   },
  // },
  // goerli: chainConnectionConfigs.goerli,
  // fuji: {
  //   ...chainConnectionConfigs.fuji,
  //   confirmations: 3,
  //   overrides: {
  //     maxFeePerGas: 500 * 10 ** 9, // 500 gwei
  //     maxPriorityFeePerGas: 10 * 10 ** 9, // 100 gwei
  //     // gasPrice: 50 * 10 ** 9, // 50 gwei
  //   },
  // },
  // fuji: chainConnectionConfigs.fuji,
  // arbitrum: chainConnectionConfigs.arbitrum,
  mumbai: chainConnectionConfigs.mumbai,
  khala: chainConnectionConfigs.khala,
  sepolia: chainConnectionConfigs.sepolia,
  // ethereum: {
  //   ...chainConnectionConfigs.ethereum,
  //   confirmations: 3,
  //   overrides: {
  //     maxFeePerGas: 150 * 10 ** 9, // gwei
  //     maxPriorityFeePerGas: 5 * 10 ** 9, // gwei
  //   },
  // },
  // moonbeam: chainConnectionConfigs.moonbeam,
};

export type MainnetChains = keyof typeof khalaConfigs;
export const chainNames = Object.keys(khalaConfigs) as MainnetChains[];
export const environment = 'khala';
