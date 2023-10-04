// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import {DomainRoutingIsm} from "../../contracts/isms/routing/DomainRoutingIsm.sol";
import {DefaultFallbackRoutingIsm} from "../../contracts/isms/routing/DefaultFallbackRoutingIsm.sol";
import {DefaultFallbackRoutingIsmFactory, DomainRoutingIsmFactory} from "../../contracts/isms/routing/DomainRoutingIsmFactory.sol";
import {IInterchainSecurityModule} from "../../contracts/interfaces/IInterchainSecurityModule.sol";
import {MessageUtils, TestIsm} from "./IsmTestUtils.sol";
import {TestMailbox} from "../../contracts/test/TestMailbox.sol";

contract DomainRoutingIsmTest is Test {
    address private constant NON_OWNER =
        0xCAfEcAfeCAfECaFeCaFecaFecaFECafECafeCaFe;
    event ModuleSet(uint32 indexed domain, IInterchainSecurityModule module);
    DomainRoutingIsm internal ism;

    function setUp() public virtual {
        ism = new DomainRoutingIsm();
        ism.initialize(address(this));
    }

    function deployTestIsm(bytes32 requiredMetadata)
        internal
        returns (TestIsm)
    {
        return new TestIsm(abi.encode(requiredMetadata));
    }

    function getMetadata(uint32 domain) internal view returns (bytes memory) {
        return TestIsm(address(ism.modules(domain))).requiredMetadata();
    }

    function testSet(uint32 domain) public {
        TestIsm _ism = deployTestIsm(bytes32(0));
        vm.expectEmit(true, true, false, true);
        emit ModuleSet(domain, _ism);
        ism.set(domain, _ism);
        assertEq(address(ism.modules(domain)), address(_ism));
    }

    function testSetManyViaFactory(uint8 count, uint32 domain) public {
        vm.assume(domain > count);
        DomainRoutingIsmFactory factory = new DomainRoutingIsmFactory();
        uint32[] memory _domains = new uint32[](count);
        IInterchainSecurityModule[]
            memory _isms = new IInterchainSecurityModule[](count);
        for (uint32 i = 0; i < count; ++i) {
            _domains[i] = domain - i;
            _isms[i] = deployTestIsm(bytes32(0));
            vm.expectEmit(true, true, false, true);
            emit ModuleSet(_domains[i], _isms[i]);
        }
        ism = factory.deploy(_domains, _isms);
        for (uint256 i = 0; i < count; ++i) {
            assertEq(address(ism.modules(_domains[i])), address(_isms[i]));
        }
    }

    function testSetNonOwner(uint32 domain, IInterchainSecurityModule _ism)
        public
    {
        vm.prank(NON_OWNER);
        vm.expectRevert("Ownable: caller is not the owner");
        ism.set(domain, _ism);
    }

    function testVerify(uint32 domain, bytes32 seed) public {
        ism.set(domain, deployTestIsm(seed));

        bytes memory metadata = getMetadata(domain);
        assertTrue(ism.verify(metadata, MessageUtils.build(domain)));
    }

    function testVerifyNoIsm(uint32 domain, bytes32 seed) public virtual {
        vm.assume(domain > 0);
        ism.set(domain, deployTestIsm(seed));

        bytes memory metadata = getMetadata(domain);
        vm.expectRevert("No ISM found for origin domain");
        ism.verify(metadata, MessageUtils.build(domain - 1));
    }

    function testRoute(uint32 domain, bytes32 seed) public {
        TestIsm testIsm = deployTestIsm(seed);
        ism.set(domain, testIsm);
        assertEq(
            address(ism.route(MessageUtils.build(domain))),
            address(testIsm)
        );
    }
}

contract DefaultFallbackRoutingIsmTest is DomainRoutingIsmTest {
    TestIsm defaultIsm;

    function setUp() public override {
        defaultIsm = deployTestIsm(bytes32(0));
        TestMailbox mailbox = new TestMailbox(1000);
        mailbox.initialize(address(this), address(defaultIsm));

        ism = new DefaultFallbackRoutingIsm(address(mailbox));
        ism.initialize(address(this));
    }

    function testConstructorReverts() public {
        vm.expectRevert("DefaultFallbackRoutingIsm: INVALID_MAILBOX");
        new DefaultFallbackRoutingIsm(address(0));
    }

    function testVerifyNoIsm(uint32 domain, bytes32 seed) public override {
        vm.assume(domain > 0);
        ism.set(domain, deployTestIsm(seed));

        bytes memory metadata = getMetadata(domain);
        bytes memory message = MessageUtils.build(domain - 1);
        vm.expectCall(
            address(defaultIsm),
            abi.encodeCall(defaultIsm.verify, (metadata, message))
        );
        ism.verify(metadata, message);
    }
}
