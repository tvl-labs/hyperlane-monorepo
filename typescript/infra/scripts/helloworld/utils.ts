import {
  HelloMultiProtocolApp,
  HelloWorldApp,
  helloWorldFactories,
} from '@hyperlane-xyz/helloworld';
import {
  AgentConnectionType,
  HyperlaneCore,
  HyperlaneIgp,
  MultiProtocolCore,
  MultiProtocolProvider,
  MultiProvider,
  attachContractsMap,
  attachContractsMapAndGetForeignDeployments,
  chainMetadata,
  filterChainMapToProtocol,
  hyperlaneEnvironments,
  igpFactories,
} from '@hyperlane-xyz/sdk';
import { ProtocolType, objMap } from '@hyperlane-xyz/utils';

import { Contexts } from '../../config/contexts';
import { EnvironmentConfig } from '../../src/config';
import { deployEnvToSdkEnv } from '../../src/config/environment';
import { HelloWorldConfig } from '../../src/config/helloworld/types';
import { Role } from '../../src/roles';
import { getKeyForRole } from '../utils';

export async function getHelloWorldApp(
  coreConfig: EnvironmentConfig,
  context: Contexts,
  keyRole: Role,
  keyContext: Contexts = context,
  connectionType: AgentConnectionType = AgentConnectionType.Http,
) {
  const multiProvider: MultiProvider = await coreConfig.getMultiProvider(
    keyContext,
    keyRole,
    connectionType,
  );
  const helloworldConfig = getHelloWorldConfig(coreConfig, context);

  const { contractsMap, foreignDeployments } =
    attachContractsMapAndGetForeignDeployments(
      helloworldConfig.addresses,
      helloWorldFactories,
      multiProvider,
    );

  const core = HyperlaneCore.fromEnvironment(
    deployEnvToSdkEnv[coreConfig.environment],
    multiProvider,
  );
  return new HelloWorldApp(
    core,
    contractsMap,
    multiProvider,
    foreignDeployments,
  );
}

export async function getHelloWorldMultiProtocolApp(
  coreConfig: EnvironmentConfig,
  context: Contexts,
  keyRole: Role,
  keyContext: Contexts = context,
  connectionType: AgentConnectionType = AgentConnectionType.Http,
) {
  const multiProvider: MultiProvider = await coreConfig.getMultiProvider(
    keyContext,
    keyRole,
    connectionType,
  );
  const sdkEnvName = deployEnvToSdkEnv[coreConfig.environment];
  const envAddresses = hyperlaneEnvironments[sdkEnvName];
  const keys = await coreConfig.getKeys(keyContext, keyRole);

  // Fetch all the keys, which is required to get the address for
  // certain cloud keys
  await Promise.all(Object.values(keys).map((key) => key.fetch()));

  const helloworldConfig = getHelloWorldConfig(coreConfig, context);

  const multiProtocolProvider =
    MultiProtocolProvider.fromMultiProvider(multiProvider);
  // Hacking around infra code limitations, we may need to add solana manually
  // because the it's not in typescript/infra/config/environments/testnet3/chains.ts
  // Adding it there breaks many things
  if (
    coreConfig.environment === 'testnet3' &&
    !multiProtocolProvider.getKnownChainNames().includes('solanadevnet')
  ) {
    multiProvider.addChain(chainMetadata.solanadevnet);
    multiProtocolProvider.addChain(chainMetadata.solanadevnet);
    keys['solanadevnet'] = getKeyForRole(
      coreConfig.environment,
      context,
      'solanadevnet',
      keyRole,
    );
    await keys['solanadevnet'].fetch();
  } else if (
    coreConfig.environment === 'mainnet2' &&
    !multiProtocolProvider.getKnownChainNames().includes('solana')
  ) {
    multiProvider.addChain(chainMetadata.solana);
    multiProtocolProvider.addChain(chainMetadata.solana);
    keys['solana'] = getKeyForRole(
      coreConfig.environment,
      context,
      'solana',
      keyRole,
    );
    await keys['solana'].fetch();
  }

  const core = MultiProtocolCore.fromAddressesMap(
    envAddresses,
    multiProtocolProvider,
  );

  const routersAndMailboxes = objMap(
    helloworldConfig.addresses,
    (chain, addresses) => ({
      router: addresses.router,
      // @ts-ignore allow loosely typed chain name to index env addresses
      mailbox: envAddresses[chain].mailbox,
    }),
  );
  const app = new HelloMultiProtocolApp(
    multiProtocolProvider,
    routersAndMailboxes,
  );

  // TODO we need a MultiProtocolIgp
  // Using an standard IGP for just evm chains for now
  // Unfortunately this requires hacking surgically around certain addresses
  const filteredAddresses = filterChainMapToProtocol(
    envAddresses,
    ProtocolType.Ethereum,
    multiProtocolProvider,
  );
  const contractsMap = attachContractsMap(filteredAddresses, igpFactories);
  const igp = new HyperlaneIgp(contractsMap, multiProvider);

  return { app, core, igp, multiProvider, multiProtocolProvider, keys };
}

export function getHelloWorldConfig(
  coreConfig: EnvironmentConfig,
  context: Contexts,
): HelloWorldConfig {
  const helloWorldConfigs = coreConfig.helloWorld;
  if (!helloWorldConfigs) {
    throw new Error(
      `Environment ${coreConfig.environment} does not have a HelloWorld config`,
    );
  }
  const config = helloWorldConfigs[context];
  if (!config) {
    throw new Error(`Context ${context} does not have a HelloWorld config`);
  }
  return config;
}
