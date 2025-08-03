// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "forge-std/Script.sol";
import "../src/EscrowFactory.sol";

contract DeployEscrowFactory is Script {
    function run() external {
        vm.startBroadcast();
        EscrowFactory factory = new EscrowFactory();
        vm.stopBroadcast();
        console.log("Factory deployed at: ", address(factory));
    }
}
