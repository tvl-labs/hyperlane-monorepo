// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;

import "forge-std/Test.sol";

import {HypNativeScaled} from "../contracts/extensions/HypNativeScaled.sol";
import {HypERC20} from "../contracts/HypERC20.sol";
import {TypeCasts} from "@hyperlane-xyz/core/contracts/libs/TypeCasts.sol";
import {MockHyperlaneEnvironment} from "@hyperlane-xyz/core/contracts/mock/MockHyperlaneEnvironment.sol";

contract HypNativeScaledTest is Test {
    uint32 nativeDomain = 1;
    uint32 synthDomain = 2;

    uint8 decimals = 9;
    uint256 mintAmount = 123456789;
    uint256 nativeDecimals = 18;
    uint256 scale = 10**(nativeDecimals - decimals);

    event Donation(address indexed sender, uint256 amount);
    event SentTransferRemote(
        uint32 indexed destination,
        bytes32 indexed recipient,
        uint256 amount
    );
    event ReceivedTransferRemote(
        uint32 indexed origin,
        bytes32 indexed recipient,
        uint256 amount
    );

    HypNativeScaled native;
    HypERC20 synth;

    MockHyperlaneEnvironment environment;

    function setUp() public {
        environment = new MockHyperlaneEnvironment(synthDomain, nativeDomain);

        synth = new HypERC20(decimals);
        synth.initialize(
            address(environment.mailboxes(synthDomain)),
            address(environment.igps(synthDomain)),
            mintAmount * (10**decimals),
            "Zebec BSC Token",
            "ZBC"
        );

        native = new HypNativeScaled(scale);
        native.initialize(
            address(environment.mailboxes(nativeDomain)),
            address(environment.igps(nativeDomain))
        );

        native.enrollRemoteRouter(
            synthDomain,
            TypeCasts.addressToBytes32(address(synth))
        );
        synth.enrollRemoteRouter(
            nativeDomain,
            TypeCasts.addressToBytes32(address(native))
        );
    }

    function test_constructor() public {
        assertEq(native.scale(), scale);
    }

    uint256 receivedValue;

    receive() external payable {
        receivedValue = msg.value;
    }

    function test_receive(uint256 amount) public {
        vm.assume(amount < address(this).balance);
        vm.expectEmit(true, true, true, true);
        emit Donation(address(this), amount);
        (bool success, bytes memory returnData) = address(native).call{
            value: amount
        }("");
        assert(success);
        assertEq(returnData.length, 0);
    }

    function test_handle(uint256 amount) public {
        vm.assume(amount <= mintAmount);

        uint256 synthAmount = amount * (10**decimals);
        uint256 nativeAmount = amount * (10**nativeDecimals);

        vm.deal(address(native), nativeAmount);

        bytes32 recipient = TypeCasts.addressToBytes32(address(this));
        synth.transferRemote(nativeDomain, recipient, synthAmount);

        vm.expectEmit(true, true, true, true);
        emit ReceivedTransferRemote(synthDomain, recipient, synthAmount);
        environment.processNextPendingMessage();

        assertEq(receivedValue, nativeAmount);
    }

    function test_handle_reverts_whenAmountExceedsSupply(uint256 amount)
        public
    {
        vm.assume(amount <= mintAmount);

        bytes32 recipient = TypeCasts.addressToBytes32(address(this));
        synth.transferRemote(nativeDomain, recipient, amount);

        uint256 nativeValue = amount * scale;
        vm.deal(address(native), nativeValue / 2);

        if (amount > 0) {
            vm.expectRevert(bytes("Address: insufficient balance"));
        }
        environment.processNextPendingMessage();
    }

    function test_tranferRemote(uint256 amount) public {
        vm.assume(amount <= mintAmount);

        uint256 nativeValue = amount * (10**nativeDecimals);
        uint256 synthAmount = amount * (10**decimals);
        address recipient = address(0xdeadbeef);
        bytes32 bRecipient = TypeCasts.addressToBytes32(recipient);

        vm.assume(nativeValue < address(this).balance);
        vm.expectEmit(true, true, true, true);
        emit SentTransferRemote(synthDomain, bRecipient, synthAmount);
        native.transferRemote{value: nativeValue}(
            synthDomain,
            bRecipient,
            nativeValue
        );
        environment.processNextPendingMessageFromDestination();
        assertEq(synth.balanceOf(recipient), synthAmount);
    }

    function test_transferRemote_reverts_whenAmountExceedsValue(
        uint256 nativeValue
    ) public {
        vm.assume(nativeValue < address(this).balance);

        address recipient = address(0xdeadbeef);
        bytes32 bRecipient = TypeCasts.addressToBytes32(recipient);
        vm.expectRevert("Native: amount exceeds msg.value");
        native.transferRemote{value: nativeValue}(
            synthDomain,
            bRecipient,
            nativeValue + 1
        );
    }
}
