use alloc::string::String;
use alloy_primitives::{Address,};
// use alloy_sol_types::sol_data::String;


pub trait IERC20 {
    fn name(&self) -> String;
    fn symbol(&self) -> String;
    fn decimals(&self) -> u8;
    fn total_supply(&self) -> u128;
    fn balance_of(&self, owner: Address) -> u128;
    fn allowance(&self, owner: Address, spender: Address) -> u128;
    fn transfer(&self, to: Address, value: u128) -> bool;
    fn transfer_from(&self, from: Address, to: Address, value: u128) -> bool;
    fn approve(&self, spender: Address, value: u128) -> bool;
}