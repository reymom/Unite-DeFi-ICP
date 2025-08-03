// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

contract EscrowSrc {
    bytes32 public orderHash;
    bytes32 public hashlock;
    address public maker;
    address public taker;
    address public token;
    uint256 public amount;
    uint256 public safetyDeposit;
    uint256 public timelocks; // packed slots

    constructor(
        bytes32 _orderHash,
        bytes32 _hashlock,
        address _maker,
        address _taker,
        address _token,
        uint256 _amount,
        uint256 _safetyDeposit,
        uint256 _timelocks
    ) {
        orderHash = _orderHash;
        hashlock = _hashlock;
        maker = _maker;
        taker = _taker;
        token = _token;
        amount = _amount;
        safetyDeposit = _safetyDeposit;
        timelocks = _timelocks;
    }

    // Withdraw/cancel are stubs -- for now just let anyone call them for testing.
    function mockWithdraw(bytes32 /*secret*/) external pure returns (bool) {
        // real implementation would transfer tokens etc.
        return true;
    }
    function mockCancel() external pure returns (bool) {
        return true;
    }
}
