// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "./EscrowSrc.sol";
import "./EscrowDst.sol";

contract EscrowFactory {
    // Deploy EscrowSrc, initialize with parameters, return address
    function deployEscrowSrc(
        bytes32 orderHash,
        bytes32 hashlock,
        address maker,
        address taker,
        address token,
        uint256 amount,
        uint256 safetyDeposit,
        uint256 timelocks
    ) external returns (address) {
        EscrowSrc src = new EscrowSrc(
            orderHash, hashlock, maker, taker, token, amount, safetyDeposit, timelocks
        );
        return address(src);
    }
    // Deploy EscrowDst, initialize with parameters, return address
    function deployEscrowDst(
        bytes32 orderHash,
        bytes32 hashlock,
        address maker,
        address taker,
        address token,
        uint256 amount,
        uint256 safetyDeposit,
        uint256 timelocks
    ) external returns (address) {
        EscrowDst dst = new EscrowDst(
            orderHash, hashlock, maker, taker, token, amount, safetyDeposit, timelocks
        );
        return address(dst);
    }

    // View helpers for deterministic address calculation
    function addressOfEscrowSrc(
        bytes32, bytes32, address, address, address, uint256, uint256, uint256
    ) external pure returns (address) {
        // For real implementation: calculate create2 address as in your client mock.
        return address(0);
    }
    function addressOfEscrowDst(
        bytes32, bytes32, address, address, address, uint256, uint256, uint256
    ) external pure returns (address) {
        // For real implementation: calculate create2 address as in your client mock.
        return address(0);
    }
}