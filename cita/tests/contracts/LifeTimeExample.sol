pragma solidity ^0.4.24;

contract LifeTimeExample {
    address public owner;
    uint storedData;
    bool public frozen;

    constructor() public payable {
        owner = msg.sender;
        frozen = false;
    }

    modifier CheckFreeze {
        require(frozen == false, "This function is frozen.");
        _;
    }

    function freeze() public {
        frozen = true;
    }

    function unfreeze() public {
        frozen = false;
    }

    function set(uint x) CheckFreeze public {
        storedData = x;
    }

    function kill() CheckFreeze public {
        selfdestruct(owner);
    }

    function get() view public returns (uint) {
        return storedData;
    }

    function() public payable {}
}
