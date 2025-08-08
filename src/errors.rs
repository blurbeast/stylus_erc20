use alloy_sol_types::sol;
use stylus_sdk::prelude::SolidityError;

sol!(
    #[derive(Debug)]
    error InsufficientBalance(address account, uint256 amount);
    #[derive(Debug)]
    error TransferToZeroAddress();
    #[derive(Debug)]
    error InsufficientAllowance(address spender, uint256 amount);

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

);

#[derive(SolidityError, Debug)]
pub enum ERC20Errors {
    InsufficientBalance(InsufficientBalance),
    TransferToZeroAddress(TransferToZeroAddress),
    InsufficientAllowance(InsufficientAllowance),
}
