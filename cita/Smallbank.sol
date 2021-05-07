pragma solidity ^0.4.24;

contract Smallbank {

    uint constant MAX_ACCOUNT = 10000;
    uint constant BALANCE = 10000;

    mapping(address => uint) savingStore;
    mapping(address => uint) checkingStore;

    event Almagate(address indexed sender, address indexed receiver);
    event UpdateBalance(address indexed addr, uint indexed tokenAmount);
    event UpdateSaving(address indexed addr, uint indexed tokenAmount);
    event SendPayment(address indexed _sender, address indexed _receiver, uint256 indexed tokenAmount);

    constructor() public {}

    function almagate(address addr0, address addr1) public {
        uint bal1 = savingStore[addr0];
        uint bal2 = checkingStore[addr1];
        checkingStore[addr0] = 0;
        savingStore[addr1] = bal1 + bal2;
        emit Almagate(addr0, addr1);
    }

    function getBalance(address _addr) public view returns (uint256) {
        uint256 bal1 = savingStore[_addr];
        uint256 bal2 = checkingStore[_addr];
        return bal1 + bal2;
    }

    function updateBalance(address addr, uint amount) public returns(bool) {
        checkingStore[addr] += amount;
        emit UpdateBalance(addr, amount);
        return true;
    }

    function updateSaving(address addr, uint amount) public returns(bool) {
        savingStore[addr] += amount;
        emit UpdateSaving(addr, amount);
        return true;
    }

    function sendPayment(address sender, address receiver, uint256 amount) public returns(bool) {
        checkingStore[sender] -= amount;
        checkingStore[receiver] += amount;
        emit SendPayment(sender, receiver, amount);
        return true;
    }
/*
    function writeCheck(address addr0, uint256 amount) public {
        uint bal1 = checkingStore[addr0];
        uint bal2 = savingStore[addr0];

        if (amount < bal1 + bal2) {
            checkingStore[addr0] = bal1 - amount - 1;
        }
        else {
            checkingStore[addr0] = bal1 - amount;
        }
    }
*/
}
