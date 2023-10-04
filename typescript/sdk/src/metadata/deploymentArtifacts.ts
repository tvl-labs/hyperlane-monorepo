import { z } from 'zod';

import { ZHash } from './customZodTypes';

export const HyperlaneDeploymentArtifactsSchema = z.object({
  mailbox: ZHash.describe('The address of the Mailbox contract.'),
  interchainGasPaymaster: ZHash.describe(
    'The address of the Interchain Gas Paymaster (IGP) contract.',
  ),
  validatorAnnounce: ZHash.describe(
    'The address of the Validator Announce contract.',
  ),
  interchainSecurityModule: ZHash.optional().describe(
    'The address of the Interchain Security Module (ISM) contract.',
  ),
});

export type HyperlaneDeploymentArtifacts = z.infer<
  typeof HyperlaneDeploymentArtifactsSchema
>;
