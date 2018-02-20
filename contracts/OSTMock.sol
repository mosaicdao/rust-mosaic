pragma solidity ^0.4.17;

// Copyright 2018 OpenST Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// 
// ----------------------------------------------------------------------------
// Common: OST Simple Token Mock contract
//
// Simple Token project: https://www.simpletoken.org/
// OpenST Foundation:    https://www.openst.org/
// OST.com:              https://www.ost.com
//
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// Based on the EIP20 token standard as specified at:
// https://github.com/ethereum/EIPs/blob/master/EIPS/eip-20-token-standard.md
// ----------------------------------------------------------------------------

import "./SafeMath.sol";
import "./OSTInterface.sol";

//
// OST Token Configuration
//

contract SimpleTokenConfig {

    string  public constant TOKEN_SYMBOL   = "ST";
    string  public constant TOKEN_NAME     = "Simple Token";
    uint8   public constant TOKEN_DECIMALS = 18;

    uint256 public constant DECIMALSFACTOR = 10**uint256(TOKEN_DECIMALS);
    uint256 public constant TOKENS_MAX     = 800000000 * DECIMALSFACTOR;
}

//
// OSTMock is a reduced version of Simple Token under the assumption
// that Simple Token is already finalized.  The initial distribution is
// still attributed to the creator of the contract.
// Further Simple Token is a standard ERC20 token with burn functionality.
//

contract OSTMock is OSTInterface, SimpleTokenConfig {
    
    using SafeMath for uint256;

    string  private tokenName;
    string  private tokenSymbol;
    uint8   private tokenDecimals;
    uint256 internal tokenTotalSupply;

    mapping(address => uint256) balances;
    mapping(address => mapping (address => uint256)) allowed;


    function OSTMock()
        public
    {
        tokenSymbol           = TOKEN_SYMBOL;
        tokenName             = TOKEN_NAME;
        tokenDecimals         = TOKEN_DECIMALS;
        tokenTotalSupply      = TOKENS_MAX;
        balances[msg.sender]  = TOKENS_MAX;

        // According to the ERC20 standard, a token contract which creates new tokens should trigger
        // a Transfer event and transfers of 0 values must also fire the event.
        Transfer(0x0, msg.sender, TOKENS_MAX);
    }


    function name() public view returns (string) {
        return tokenName;
    }


    function symbol() public view returns (string) {
        return tokenSymbol;
    }


    function decimals() public view returns (uint8) {
        return tokenDecimals;
    }


    function totalSupply() public view returns (uint256) {
        return tokenTotalSupply;
    }


    function balanceOf(address _owner) public view returns (uint256) {
        return balances[_owner];
    }


    function allowance(address _owner, address _spender) public view returns (uint256 remaining) {
        return allowed[_owner][_spender];
    }


    function transfer(address _to, uint256 _value) public returns (bool success) {
        // According to the EIP20 spec, "transfers of 0 values MUST be treated as normal
        // transfers and fire the Transfer event".
        // Also, should throw if not enough balance. This is taken care of by SafeMath.
        balances[msg.sender] = balances[msg.sender].sub(_value);
        balances[_to] = balances[_to].add(_value);

        Transfer(msg.sender, _to, _value);

        return true;
    }


    function transferFrom(address _from, address _to, uint256 _value) public returns (bool success) {
        balances[_from] = balances[_from].sub(_value);
        allowed[_from][msg.sender] = allowed[_from][msg.sender].sub(_value);
        balances[_to] = balances[_to].add(_value);

        Transfer(_from, _to, _value);

        return true;
    }


    function approve(address _spender, uint256 _value) public returns (bool success) {

        allowed[msg.sender][_spender] = _value;

        Approval(msg.sender, _spender, _value);

        return true;
    }


    // Implement a burn function to permit msg.sender to reduce its balance
    // which also reduces tokenTotalSupply
    function burn(uint256 _value) public returns (bool success) {
        require(_value <= balances[msg.sender]);

        balances[msg.sender] = balances[msg.sender].sub(_value);
        tokenTotalSupply = tokenTotalSupply.sub(_value);

        Burnt(msg.sender, _value);

        return true;
    }
}
