pragma solidity ^0.4.24;

/// @title The interface of version manager
/// @author ["Cryptape Technologies <contact@cryptape.com>"]
interface IVersionManager {
    /// @notice Deprecated. Check the setProtocolVersion
    function setVersion(uint32 _version) external;
    function setProtocolVersion(uint32 _version) external;
    /// @notice Deprecated. Check the setProtocolVersion
    function getVersion() external returns (uint32);
    function getProtocolVersion() external returns (uint32);
}
