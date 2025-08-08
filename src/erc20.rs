
use crate::{errors, ierc20};
use alloc::string::{String, };
use alloc::vec::Vec;
use ierc20::IERC20;
use errors::ERC20Errors;
use stylus_sdk::storage::{StorageAddress, StorageMap, StorageString, StorageU256};
use stylus_sdk::{alloy_primitives::{Address, U256}, prelude::*, ArbResult};
use crate::errors::{InsufficientAllowance, InsufficientBalance};

#[storage]
pub struct ERC20 {
    name: StorageString,
    symbol: StorageString,
    owner: StorageAddress,
    total_supply: StorageU256,
    balance: StorageMap<Address, StorageU256>,
    allowance: StorageMap<Address, StorageMap<Address, StorageU256>>,
}

#[public]
impl IERC20 for ERC20 {

    fn init(&mut self, name: String, symbol: String, initial_supply: u128) -> Result<bool, Vec<u8>> {
            self.name.set_str(name);
            self.symbol.set_str(symbol);
            self._mint(self.vm().msg_sender(), U256::from(initial_supply));
            self.owner.set(self.vm().msg_sender());
        Ok(true)
    }

    fn name(&self) -> String {
        self.name.get_string()
    }

    fn symbol(&self) -> String {
        self.symbol.get_string()
    }

    fn decimals(&self) -> u8 {
        18
    }

    fn total_supply(&self) -> u128 {
        self.total_supply.get().try_into().unwrap()
    }

    fn balance_of(&self, owner: Address) -> u128 {
        self._balance(owner).try_into().unwrap()
    }

    fn allowance(&self, owner: Address, spender: Address) -> u128 {
        self._get_allowance(owner, spender).try_into().unwrap()
    }

    fn transfer(&mut self, to: Address, value: u128) -> ArbResult {
        let from: Address = self.vm().msg_sender();
        let bal: U256 = self._balance(from);
        self._check_balance(bal, value.try_into().unwrap()).ok_or(self._throw_insufficient_balance(from, value.try_into().unwrap()))?;
        self._transfer(from, bal, to, value.try_into().unwrap());
        Ok(vec![true.into()])
    }

    fn transfer_from(&mut self, owner: Address, to: Address, value: u128) ->ArbResult {
        let spender: Address = self.vm().msg_sender();
        let allowance: U256 = self._get_allowance(owner, spender);

        if self._check_allowance(allowance, U256::from(value)).is_none(){
            return Err(self._throw_insufficient_allowance(spender, allowance).try_into().unwrap());
        }

        let owner_balance = self.balance.get(owner);

        if self._check_balance(owner_balance, value.try_into().unwrap()).is_none() {
            return Err(self._throw_insufficient_balance(owner,  allowance).try_into().unwrap());
        }
        self._update_allowance(owner, spender, allowance, value.try_into().unwrap());
        self._transfer(owner, owner_balance, to, U256::from(value));
        Ok(vec![true.into()])
    }

    fn approve(&mut self, spender: Address, value: u128) -> ArbResult {
        let from: Address = self.vm().msg_sender();
        // check balance
        self._check_balance(self._balance(from), value.try_into().unwrap()).ok_or(self._throw_insufficient_balance(from, value.try_into().unwrap()))?;
        self.allowance.setter(from).setter(spender).set(value.try_into().unwrap());
        Ok(vec![true.into()])
    }
}

impl ERC20 {

    fn _throw_insufficient_balance(&self, address: Address, value: U256) -> ERC20Errors {
        ERC20Errors::InsufficientBalance(InsufficientBalance{
            account: address,
            amount: value
        })
    }

    fn _throw_insufficient_allowance(&self, spender: Address, value: U256) -> ERC20Errors {
        ERC20Errors::InsufficientAllowance(InsufficientAllowance{
            spender,
            amount: value
        })
    }
}

impl ERC20 {
    fn _transfer(&mut self, from: Address, bal: U256, to: Address, value: U256) -> bool {
        self.balance.insert(from, bal - value);
        self.balance.insert(to, self.balance.get(to) + value);
        true
    }

    fn _get_allowance(&self, owner: Address, spender: Address) -> U256 {
        self.allowance.get(owner).get(spender)
    }
    fn _update_allowance(&mut self, owner: Address, spender: Address, allowance: U256, value: U256) {
        self.allowance.setter(owner).setter(spender).set(allowance-value);
    }

    fn _check_allowance(&self, allowance: U256, amount: U256) -> Option<bool> {
        if allowance >= amount{
            return Some(true);
        }
        None
    }

    fn _check_balance(&self, balance: U256, value: U256 ) -> Option<bool> {
        //check balance
        if balance >= value {
            return Some(true);
        }
        None
    }

    fn _mint(&mut self, to: Address, value: U256) {
        let bal = self.balance.get(to);
        self.balance.insert(to, bal+value);
        self.total_supply.set(self.total_supply.get()+value);
    }
    fn _balance(&self, address: Address) -> U256 {
        self.balance.get(address)
    }
}

#[cfg(test)]
mod test {
    // use alloy_primitives::U160;
    // use super::*;
    // use stylus_sdk::{testing::*, alloy_primitives::{U256, Address, }};
    //
    // fn create_erc20_instance() -> (ERC20, Address, TestVM) {
    //     let vm: TestVM = TestVM::default();
    //     let mut erc20 = ERC20::from(&vm);
    //     erc20.init();
    //     (erc20, vm.msg_sender(), vm)
    // }
    //
    // #[test]
    // fn test_get_decimals() {
    //     let (erc20, owner, _) = create_erc20_instance();
    //     assert_eq!(erc20.decimals(), 18);
    //     assert_eq!(erc20.name.get_string(), "Stylus");
    //     assert_eq!(erc20.owner.get(), owner);
    //     assert_eq!(erc20.total_supply.get(), U256::from(1_000_000_000));
    //     assert_eq!(erc20.balance.get(owner), U256::from(1_000_000_000));
    // }
    //
    // #[test]
    // // #[should_panic(expected = "[110, 111, 116, 32, 114, 101, 97, 100, 121]")]
    // fn test_transfer() {
    //     let (mut erc20, owner, _) = create_erc20_instance();
    //     let to: Address = Address::from(U160::from(0x0000000000000000000000000000000000000001));
    //
    //     erc20.transfer(to, U256::from(100)).unwrap();
    //     assert_eq!(erc20.balance.get(owner), U256::from(999_999_900));
    //     assert_eq!(erc20.balance.get(to), U256::from(100));
    // }
    //
    // #[test]
    // #[should_panic]
    // fn test_transfer_error() {
    //     let (mut erc20, owner, vm) = create_erc20_instance();
    //     let to: Address = Address::from(U160::from(0x0000000000000000000000000000000000000001));
    //     // cannot transfer to self
    //     erc20.transfer(owner, U256::from(100)).unwrap();
    //
    //     // cannot send 0
    //     // erc20.transfer(to, U256::from(0)).unwrap();
    //     assert_eq!(erc20.balance.get(owner), U256::from(1_000_000_000));
    //
    //     //cannot transfer from 0 balance
    //     vm.set_sender(to);
    //     erc20.transfer(owner, U256::from(90)).unwrap();
    //
    // }
    //
    // #[test]
    // fn test_transfer_from_and_approve() {
    //     let (mut erc20, owner, vm) = create_erc20_instance();
    //     let to: Address = Address::from(U160::from(0x0000000000000000000000000000000000000001));
    //
    //     erc20.approve(to, U256::from(100)).unwrap();
    //     assert_eq!(erc20.allowance.get(owner).get(to), U256::from(100));
    //
    //     vm.set_sender(to);
    //     erc20.transfer_from(owner, vm.contract_address(), U256::from(100)).unwrap();
    //     assert_eq!(erc20.balance.get(owner), U256::from(999_999_900));
    //     assert_eq!(erc20.balance.get(vm.contract_address()), U256::from(100));
    //     assert_eq!(erc20.balance.get(to), U256::from(0));
    // }
}
