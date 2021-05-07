pragma solidity ^0.4.24;

contract TestCase {
    uint a;
    uint b;
    uint c;
    event Init(address, uint);
    event Set(address, uint);
    event Stored(uint);
    

    constructor() public {
        a = 100;
	b = 100;
        emit Init(msg.sender, 100);
    }

    function r_a_w_a(uint x) public {
        a = a + x;
    }
    function r_b_w_b(uint x) public {
        b = b + x;
    }
    function r_ab_w_a(uint x) public {        
	a = b + x;
	a = x;
    }
    function set_a(uint x) public {
        emit Stored(x);
        a = x;
        emit Set(msg.sender, x);
    }
    function set_b(uint x) public {
        emit Stored(x);
        b = x;
        emit Set(msg.sender, x);
    }
}
