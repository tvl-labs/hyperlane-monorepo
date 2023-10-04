import { expect } from 'chai';
import { ethers } from 'hardhat';

import {
  addressToBytes32,
  formatMessage,
  messageId,
} from '@hyperlane-xyz/utils';

import testCases from '../../vectors/message.json';
import { TestMessage, TestMessage__factory } from '../types';

const remoteDomain = 1000;
const localDomain = 2000;
const version = 0;
const nonce = 11;

describe('Message', async () => {
  let messageLib: TestMessage;

  before(async () => {
    const [signer] = await ethers.getSigners();

    const Message = new TestMessage__factory(signer);
    messageLib = await Message.deploy();
  });

  it('Returns fields from a message', async () => {
    const [sender, recipient] = await ethers.getSigners();
    const body = ethers.utils.formatBytes32String('message');

    const message = formatMessage(
      version,
      nonce,
      remoteDomain,
      sender.address,
      localDomain,
      recipient.address,
      body,
    );

    expect(await messageLib.version(message)).to.equal(version);
    expect(await messageLib.nonce(message)).to.equal(nonce);
    expect(await messageLib.origin(message)).to.equal(remoteDomain);
    expect(await messageLib.sender(message)).to.equal(
      addressToBytes32(sender.address),
    );
    expect(await messageLib.destination(message)).to.equal(localDomain);
    expect(await messageLib.recipient(message)).to.equal(
      addressToBytes32(recipient.address),
    );
    expect(await messageLib.recipientAddress(message)).to.equal(
      recipient.address,
    );
    expect(await messageLib.body(message)).to.equal(body);
  });

  it('Matches Rust-output HyperlaneMessage and leaf', async () => {
    for (const test of testCases) {
      const { origin, sender, destination, recipient, body, nonce, id } = test;

      const hexBody = ethers.utils.hexlify(body);

      const hyperlaneMessage = formatMessage(
        version,
        nonce,
        origin,
        sender,
        destination,
        recipient,
        hexBody,
      );

      expect(await messageLib.origin(hyperlaneMessage)).to.equal(origin);
      expect(await messageLib.sender(hyperlaneMessage)).to.equal(sender);
      expect(await messageLib.destination(hyperlaneMessage)).to.equal(
        destination,
      );
      expect(await messageLib.recipient(hyperlaneMessage)).to.equal(recipient);
      expect(await messageLib.body(hyperlaneMessage)).to.equal(hexBody);
      expect(messageId(hyperlaneMessage)).to.equal(id);
    }
  });
});
