import {
  ChainValidatorSets,
  CheckpointSyncerType,
} from '../../../src/config/agent';

import { MainnetChains } from './chains';

const s3BucketRegion = 'us-east-1';

const s3BucketName = (index: number) => `khala-validators-signatures-${index}`;

export const validators: ChainValidatorSets<MainnetChains> = {
  khala: {
    threshold: 1,
    validators: [
      {
        address: '0xa22f3424a39da34676d766d3dbc340a871536d78',
        name: s3BucketName(1),
        checkpointSyncer: {
          type: CheckpointSyncerType.S3,
          bucket: s3BucketName(1),
          region: s3BucketRegion,
        },
      },
    ],
  },
  fuji: {
    threshold: 1,
    validators: [
      {
        address: '0xc27faa511c23e24b365a9c76cbb425a4e32bfc6e',
        name: s3BucketName(2),
        checkpointSyncer: {
          type: CheckpointSyncerType.S3,
          bucket: s3BucketName(2),
          region: s3BucketRegion,
        },
      },
    ],
  },
  goerli: {
    threshold: 1,
    validators: [
      {
        address: '0x20b2f6a81f786ad3ad39e6ada4bbc3abc90e96dc',
        name: s3BucketName(3),
        checkpointSyncer: {
          type: CheckpointSyncerType.S3,
          bucket: s3BucketName(3),
          region: s3BucketRegion,
        },
      },
    ],
  },
};
