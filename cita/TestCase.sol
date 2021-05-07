pragma solidity ^0.4.24;

contract TestCase {
    uint storedData;
    event Init(address, uint);
    event Set(address, uint);
    event Stored(uint);
    uint y;

    constructor() public {
        storedData = 100;
        emit Init(msg.sender, 100);
    }

    function set(uint x) public {
        emit Stored(x);
        storedData = x;
        emit Set(msg.sender, x);
    }
    function set_1(uint x) public {
        emit Stored(x);
        y = x;
        emit Set(msg.sender, x);
    }

    function get() public constant returns (uint) {
        return storedData;
    }
}
