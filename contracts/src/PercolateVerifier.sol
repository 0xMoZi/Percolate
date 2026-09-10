// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";

struct Journal {
    bytes32 nullifier;
    bytes32 merkle_root;
    address caller_binding;
    uint64 valid_until;
}

contract PercolateVerifier {
    error NotMaker();
    error RootMismatch();
    error WrongCaller();
    error NullifierAlreadyUsed();
    error ProofExpired();

    event RootUpdated(bytes32 indexed newRoot);
    event ProofConsumed(bytes32 indexed nullifier, address indexed taker);

    address public immutable maker;
    IRiscZeroVerifier public immutable router;
    bytes32 public immutable imageId;

    bytes32 public currentRoot;
    mapping(bytes32 => bool) public nullifierUsed;

    constructor(address _maker, IRiscZeroVerifier _router, bytes32 _imageId) {
        maker = _maker;
        router = _router;
        imageId = _imageId;
    }

    function setRoot(bytes32 newRoot) external {
        if (msg.sender != maker) revert NotMaker();
        currentRoot = newRoot;
         emit RootUpdated(newRoot);
    }

    function verifyAndConsume(address taker, bytes calldata journalBytes, bytes calldata seal) external {
        Journal memory j = abi.decode(journalBytes, (Journal));

        if (j.merkle_root != currentRoot) revert RootMismatch();
        if (j.caller_binding != taker) revert WrongCaller();
        if (block.timestamp > j.valid_until) revert ProofExpired();
        if (nullifierUsed[j.nullifier]) revert NullifierAlreadyUsed();

        router.verify(seal, imageId, sha256(journalBytes));

        nullifierUsed[j.nullifier] = true;
        emit ProofConsumed(j.nullifier, taker);
    }
}
