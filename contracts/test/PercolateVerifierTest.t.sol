// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

import {Test} from "forge-std/Test.sol";
import {RiscZeroMockVerifier} from "risc0/test/RiscZeroMockVerifier.sol";
import {PercolateVerifier, Journal} from "../src/PercolateVerifier.sol";

contract PercolateVerifierTest is Test {
    RiscZeroMockVerifier mockVerifier;
    PercolateVerifier percolate;

    address maker = address(0xA11CE);
    address taker = address(0xBEEF);
    bytes32 constant IMAGE_ID = bytes32(uint256(1));

    function setUp() public {
        mockVerifier = new RiscZeroMockVerifier(bytes4(0));
        percolate = new PercolateVerifier(maker, mockVerifier, IMAGE_ID);
    }

    function _mockJournalAndSeal(Journal memory j) internal view returns (bytes memory journalBytes, bytes memory seal) {
        journalBytes = abi.encode(j);
        seal = mockVerifier.mockProve(IMAGE_ID, sha256(journalBytes)).seal;
    }

    function test_setRoot_onlyMaker() public {
        vm.prank(maker);
        percolate.setRoot(bytes32(uint256(123)));
        assertEq(percolate.currentRoot(), bytes32(uint256(123)));

        vm.prank(taker);
        vm.expectRevert(PercolateVerifier.NotMaker.selector);
        percolate.setRoot(bytes32(uint256(456)));
    }

    function test_verifyAndConsume_success() public {
        bytes32 root = bytes32(uint256(123));
        vm.prank(maker);
        percolate.setRoot(root);

        Journal memory j = Journal({
            nullifier: bytes32(uint256(1)),
            merkle_root: root,
            caller_binding: taker,
            valid_until: uint64(block.timestamp + 1 hours)
        });
        (bytes memory journalBytes, bytes memory seal) = _mockJournalAndSeal(j);

        percolate.verifyAndConsume(taker, journalBytes, seal);
        assertTrue(percolate.nullifierUsed(j.nullifier));
    }

    function test_verifyAndConsume_rejectsRootMismatch() public {
        vm.prank(maker);
        percolate.setRoot(bytes32(uint256(999)));

        Journal memory j = Journal({
            nullifier: bytes32(uint256(1)),
            merkle_root: bytes32(uint256(123)),
            caller_binding: taker,
            valid_until: uint64(block.timestamp + 1 hours)
        });
        (bytes memory journalBytes, bytes memory seal) = _mockJournalAndSeal(j);

        vm.expectRevert(PercolateVerifier.RootMismatch.selector);
        percolate.verifyAndConsume(taker, journalBytes, seal);
    }

    function test_verifyAndConsume_rejectsReplayedNullifier() public {
        bytes32 root = bytes32(uint256(123));
        vm.prank(maker);
        percolate.setRoot(root);

        Journal memory j = Journal({
            nullifier: bytes32(uint256(1)),
            merkle_root: root,
            caller_binding: taker,
            valid_until: uint64(block.timestamp + 1 hours)
        });
        (bytes memory journalBytes, bytes memory seal) = _mockJournalAndSeal(j);

        percolate.verifyAndConsume(taker, journalBytes, seal);

        vm.expectRevert(PercolateVerifier.NullifierAlreadyUsed.selector);
        percolate.verifyAndConsume(taker, journalBytes, seal);
    }

    function test_verifyAndConsume_rejectsExpiredProof() public {
        bytes32 root = bytes32(uint256(123));
        vm.prank(maker);
        percolate.setRoot(root);

        Journal memory j = Journal({
            nullifier: bytes32(uint256(1)),
            merkle_root: root,
            caller_binding: taker,
            valid_until: uint64(block.timestamp - 1)
        });
        (bytes memory journalBytes, bytes memory seal) = _mockJournalAndSeal(j);

        vm.expectRevert(PercolateVerifier.ProofExpired.selector);
        percolate.verifyAndConsume(taker, journalBytes, seal);
    }
}
