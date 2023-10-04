import { ChainName, chainMetadata } from '@hyperlane-xyz/sdk';
import { ProtocolType } from '@hyperlane-xyz/utils';

import { Contexts } from '../../config/contexts';
import {
  AgentContextConfig,
  DeployEnvironment,
  RootAgentConfig,
} from '../config';
import { Role } from '../roles';
import { fetchGCPSecret, setGCPSecret } from '../utils/gcloud';
import { execCmd } from '../utils/utils';

import { AgentAwsKey } from './aws/key';
import { AgentGCPKey } from './gcp';
import { CloudAgentKey } from './keys';

interface KeyAsAddress {
  identifier: string;
  address: string;
}

export function getRelayerCloudAgentKeys(
  agentConfig: AgentContextConfig,
): Array<CloudAgentKey> {
  if (!agentConfig.aws) {
    return [
      new AgentGCPKey(agentConfig.runEnv, agentConfig.context, Role.Relayer),
    ];
  }

  const keys = [];
  keys.push(new AgentAwsKey(agentConfig, Role.Relayer));
  const nonEthereumChains = agentConfig.contextChainNames[Role.Relayer].find(
    (chainName) => chainMetadata[chainName].protocol !== ProtocolType.Ethereum,
  );
  // If there are any non-ethereum chains, we also want hex keys.
  if (nonEthereumChains) {
    keys.push(
      new AgentGCPKey(agentConfig.runEnv, agentConfig.context, Role.Relayer),
    );
  }
  return keys;
}

// If getting all keys for relayers or validators, it's recommended to use
// `getRelayerCloudAgentKeys` or `getValidatorCloudAgentKeys` instead.
export function getCloudAgentKey(
  agentConfig: AgentContextConfig,
  role: Role,
  chainName?: ChainName,
  index?: number,
): CloudAgentKey {
  // The deployer is always GCP-based
  if (!!agentConfig.aws && role !== Role.Deployer) {
    return new AgentAwsKey(agentConfig, role, chainName, index);
  } else {
    return new AgentGCPKey(
      agentConfig.runEnv,
      agentConfig.context,
      role,
      chainName,
      index,
    );
  }
}

export function getValidatorCloudAgentKeys(
  agentConfig: RootAgentConfig,
): Array<CloudAgentKey> {
  // For each chainName, create validatorCount keys
  if (!agentConfig.validators) return [];
  const validators = agentConfig.validators;
  return agentConfig.contextChainNames[Role.Validator]
    .filter((chainName) => !!validators.chains[chainName])
    .flatMap((chainName) =>
      validators.chains[chainName].validators.map((_, index) =>
        getCloudAgentKey(agentConfig, Role.Validator, chainName, index),
      ),
    );
}

export function getAllCloudAgentKeys(
  agentConfig: RootAgentConfig,
): Array<CloudAgentKey> {
  const keys = [];
  if ((agentConfig.rolesWithKeys ?? []).includes(Role.Relayer))
    keys.push(...getRelayerCloudAgentKeys(agentConfig));
  if ((agentConfig.rolesWithKeys ?? []).includes(Role.Validator))
    keys.push(...getValidatorCloudAgentKeys(agentConfig));
  for (const role of agentConfig.rolesWithKeys) {
    if (role == Role.Relayer || role == Role.Validator) continue;
    keys.push(getCloudAgentKey(agentConfig, role));
  }
  return keys;
}

export async function deleteAgentKeys(agentConfig: AgentContextConfig) {
  const keys = getAllCloudAgentKeys(agentConfig);
  await Promise.all(keys.map((key) => key.delete()));
  await execCmd(
    `gcloud secrets delete ${addressesIdentifier(
      agentConfig.runEnv,
      agentConfig.context,
    )} --quiet`,
  );
}

export async function createAgentKeysIfNotExists(
  agentConfig: AgentContextConfig,
) {
  const keys = getAllCloudAgentKeys(agentConfig);

  await Promise.all(
    keys.map(async (key) => {
      return key.createIfNotExists();
    }),
  );

  await persistAddresses(
    agentConfig.runEnv,
    agentConfig.context,
    keys.map((key) => key.serializeAsAddress()),
  );
}

export async function rotateKey(
  agentConfig: AgentContextConfig,
  role: Role,
  chainName: ChainName,
) {
  const key = getCloudAgentKey(agentConfig, role, chainName);
  await key.update();
  const keyIdentifier = key.identifier;
  const addresses = await fetchGCPKeyAddresses(
    agentConfig.runEnv,
    agentConfig.context,
  );
  const filteredAddresses = addresses.filter((_) => {
    return _.identifier !== keyIdentifier;
  });

  filteredAddresses.push(key.serializeAsAddress());
  await persistAddresses(
    agentConfig.runEnv,
    agentConfig.context,
    filteredAddresses,
  );
}

async function persistAddresses(
  environment: DeployEnvironment,
  context: Contexts,
  keys: KeyAsAddress[],
) {
  await setGCPSecret(
    addressesIdentifier(environment, context),
    JSON.stringify(keys),
    {
      environment,
      context,
    },
  );
}

// This function returns all keys for a given mailbox chain in a dictionary where the key is the identifier
export async function fetchKeysForChain(
  agentConfig: RootAgentConfig,
  chainNames: ChainName | ChainName[],
): Promise<Record<string, CloudAgentKey>> {
  if (!Array.isArray(chainNames)) chainNames = [chainNames];

  // Get all keys for the chainNames. Include keys where chainNames is undefined,
  // which are keys that are not chain-specific but should still be included
  const keys = await Promise.all(
    getAllCloudAgentKeys(agentConfig)
      .filter(
        (key) =>
          key.chainName === undefined || chainNames.includes(key.chainName),
      )
      .map(async (key) => {
        await key.fetch();
        return [key.identifier, key];
      }),
  );

  return Object.fromEntries(keys);
}

async function fetchGCPKeyAddresses(
  environment: DeployEnvironment,
  context: Contexts,
) {
  const addresses = await fetchGCPSecret(
    addressesIdentifier(environment, context),
  );
  return addresses as KeyAsAddress[];
}

function addressesIdentifier(
  environment: DeployEnvironment,
  context: Contexts,
) {
  return `${context}-${environment}-key-addresses`;
}
