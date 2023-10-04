import { ethers } from 'ethers';

import { TestMailbox, TestMailbox__factory } from '@hyperlane-xyz/core';
import { messageId } from '@hyperlane-xyz/utils';

import { HyperlaneContracts } from '../contracts/types';
import { ChainName } from '../types';

import { HyperlaneCore } from './HyperlaneCore';
import { coreFactories } from './contracts';

export const testCoreFactories = {
  ...coreFactories,
  mailbox: new TestMailbox__factory(),
};

export class TestCoreApp extends HyperlaneCore {
  getContracts(chain: ChainName): HyperlaneContracts<typeof testCoreFactories> {
    return super.getContracts(chain) as HyperlaneContracts<
      typeof testCoreFactories
    >;
  }

  async processMessages(): Promise<
    Map<ChainName, Map<ChainName, ethers.providers.TransactionResponse[]>>
  > {
    const responses = new Map();
    for (const origin of this.chains()) {
      const outbound = await this.processOutboundMessages(origin);
      const originResponses = new Map();
      this.remoteChains(origin).forEach((destination) =>
        originResponses.set(destination, outbound.get(destination)),
      );
      responses.set(origin, originResponses);
    }
    return responses;
  }

  async processOutboundMessages(
    origin: ChainName,
  ): Promise<Map<ChainName, ethers.providers.TransactionResponse[]>> {
    const responses = new Map<ChainName, any>();
    const contracts = this.getContracts(origin);
    const outbox = contracts.mailbox as TestMailbox;

    const dispatchFilter = outbox.filters.Dispatch();
    const dispatches = await outbox.queryFilter(dispatchFilter);
    for (const dispatch of dispatches) {
      const destination = dispatch.args.destination;
      if (destination === this.multiProvider.getDomainId(origin)) {
        throw new Error('Dispatched message to local domain');
      }
      const destinationChain = this.multiProvider.getChainName(destination);
      const inbox = this.getContracts(destinationChain).mailbox;
      const id = messageId(dispatch.args.message);
      const delivered = await inbox.delivered(id);
      if (!delivered) {
        const response = await inbox.process('0x', dispatch.args.message);
        const destinationResponses = responses.get(destinationChain) || [];
        destinationResponses.push(response);
        responses.set(destinationChain, destinationResponses);
      }
    }
    return responses;
  }
}
