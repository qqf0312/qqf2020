pragma solidity ^0.4.0;

contract SmallBank {
    
    //uint constant MAX_ACCOUNT = 10000;
    //uint constant BALANCE = 10000;
    //bytes20 constant accountTab = "account";
    //bytes20 constant savingTab = "saving";
    //bytes20 constant checkingTab = "checking";
    
    mapping(string=>uint) savingStore;
    //mapping(string=>uint) checkingStore;

    function sendPayment(string arg0, string arg1, uint arg2) public {
        uint bal1 = savingStore[arg0];
        uint bal2 = savingStore[arg1];
        uint amount = arg2;
        
        bal1 -= amount;
        bal2 += amount;
        
        savingStore[arg0] = bal1;
        savingStore[arg1] = bal2;
    }
}