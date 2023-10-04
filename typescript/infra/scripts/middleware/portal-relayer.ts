import path from 'path';

import {
  LiquidityLayerApp,
  attachContractsMap,
  liquidityLayerFactories,
} from '@hyperlane-xyz/sdk';
import { error, log } from '@hyperlane-xyz/utils';

import { bridgeAdapterConfigs } from '../../config/environments/testnet3/token-bridge';
import { readJSON, sleep } from '../../src/utils/utils';
import {
  getArgs,
  getEnvironmentConfig,
  getEnvironmentDirectory,
} from '../utils';

async function relayPortalTransfers() {
  const { environment } = await getArgs().argv;
  const config = getEnvironmentConfig(environment);
  const multiProvider = await config.getMultiProvider();
  const dir = path.join(
    __dirname,
    '../../',
    getEnvironmentDirectory(environment),
    'middleware/liquidity-layer',
  );
  const addresses = readJSON(dir, 'addresses.json');
  const contracts = attachContractsMap(addresses, liquidityLayerFactories);
  const app = new LiquidityLayerApp(
    contracts,
    multiProvider,
    bridgeAdapterConfigs,
  );

  const tick = async () => {
    for (const chain of Object.keys(bridgeAdapterConfigs)) {
      log('Processing chain', {
        chain,
      });

      const txHashes = await app.fetchPortalBridgeTransactions(chain);
      const portalMessages = (
        await Promise.all(
          txHashes.map((txHash) => app.parsePortalMessages(chain, txHash)),
        )
      ).flat();

      log('Portal messages', {
        portalMessages,
      });

      // Poll for attestation data and submit
      for (const message of portalMessages) {
        try {
          await app.attemptPortalTransferCompletion(message);
        } catch (err) {
          error('Error attempting portal transfer', {
            message,
            err,
          });
        }
      }
      await sleep(10000);
    }
  };

  while (true) {
    try {
      await tick();
    } catch (err) {
      error('Error processing chains in tick', {
        err,
      });
    }
  }
}

relayPortalTransfers().then(console.log).catch(console.error);
