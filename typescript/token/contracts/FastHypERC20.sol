// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;

import {HypERC20} from "./HypERC20.sol";

import {TokenRouter} from "./libs/TokenRouter.sol";
import {FastTokenRouter} from "./libs/FastTokenRouter.sol";
import {Message} from "./libs/Message.sol";

import {ERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";

/**
 * @title Hyperlane ERC20 Token Router that extends ERC20 with remote transfer functionality.
 * @author Abacus Works
 * @dev Supply on each chain is not constant but the aggregate supply across all chains is.
 */
contract FastHypERC20 is FastTokenRouter, HypERC20 {
    constructor(uint8 __decimals) HypERC20(__decimals) {}

    /**
     * @dev delegates transfer logic to `_transferTo`.
     * @inheritdoc TokenRouter
     */
    function _handle(
        uint32 _origin,
        bytes32 _sender,
        bytes calldata _message
    ) internal virtual override(FastTokenRouter, TokenRouter) {
        FastTokenRouter._handle(_origin, _sender, _message);
    }

    /**
     * @dev Mints `_amount` of tokens to `_recipient`.
     * @inheritdoc FastTokenRouter
     */
    function _fastTransferTo(address _recipient, uint256 _amount)
        internal
        override
    {
        _mint(_recipient, _amount);
    }

    /**
     * @dev Burns `_amount` of tokens from `_recipient`.
     * @inheritdoc FastTokenRouter
     */
    function _fastRecieveFrom(address _sender, uint256 _amount)
        internal
        override
    {
        _burn(_sender, _amount);
    }
}
