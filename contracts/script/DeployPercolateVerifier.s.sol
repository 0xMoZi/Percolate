// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

import {Script, console} from "forge-std/Script.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {PercolateVerifier} from "../src/PercolateVerifier.sol";

contract DeployPercolateVerifier is Script {
    function run() external {
        address routerAddr = vm.envAddress("RISC_ZERO_ROUTER");
        address maker = vm.envAddress("MAKER_ADDRESS");
        bytes32 imageId = vm.envBytes32("IMAGE_ID");

        vm.startBroadcast();
        PercolateVerifier percolate = new PercolateVerifier(
            maker,
            IRiscZeroVerifier(routerAddr),
            imageId
        );
        vm.stopBroadcast();

        console.log("PercolateVerifier deployed at:", address(percolate));
    }
}
