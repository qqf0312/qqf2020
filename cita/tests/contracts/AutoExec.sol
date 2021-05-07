pragma solidity ^0.4.24;

contract AutoExec {
    uint public x;
    address public coinBase;

    function autoExec() public {
        x++;
        coinBase = block.coinbase;
    }
}
