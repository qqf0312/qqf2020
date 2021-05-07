pragma solidity ^0.4.24;

/// @title The interface of emergency intervention
/// @author ["Cryptape Technologies <contact@cryptape.com>"]
interface IEmergencyIntervention {
    function setState(bool _state) external;
}
