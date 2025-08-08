use alloc::string::String;
use alloc::vec::Vec;

use stylus_sdk::{prelude::*, alloy_primitives::{U256, Address, }, ArbResult};
use stylus_sdk::storage::{StorageAddress, StorageMap, StorageString, StorageU256};

#[storage]
#[entrypoint]
pub struct ERC20 {
    name: StorageString,
    symbol: StorageString,
    owner: StorageAddress,
    total_supply: StorageU256,
    balance: StorageMap<Address, StorageU256>,
    allowance: StorageMap<Address, StorageMap<Address, StorageU256>>,
}

#[public]
impl ERC20  {

    fn initialize(&mut self, name: String, symbol: String, value: U256) {
        self.name.set_str(name);
        self.symbol.set_str(symbol);
        self.mint(self.vm().msg_sender(), value);
        self.owner.set(self.vm().msg_sender());
    }
    fn init(&mut self) {
        self.name.set_str("Stylus");
        self.symbol.set_str("STY");
        self.mint(self.vm().msg_sender(), U256::from(1_000_000_000));
        self.owner.set(self.vm().msg_sender());
    }
    fn decimals(&self) -> u8 {
        18
    }
    fn approve(&mut self, spender: Address, amount: U256) -> ArbResult {
        let from: Address = self.vm().msg_sender();
        self.allowance.setter(from).setter(spender).set(amount);
        Ok(vec![true.into()])
    }

    fn transfer_from(&mut self, owner: Address, to: Address, value: U256) -> Result<bool, Vec<u8>> {
        let from = self.vm().msg_sender();
        let allowance = self.allowance.get(owner).get(from);
        assert!(allowance > U256::from(0), "insufficient allowance");
        assert!(value <= allowance, "insufficient allowance");
        self.allowance.setter(from).setter(to).set(allowance-value);
        let owner_balance = self.balance.get(owner);
        self._transfer(owner, owner_balance, to, value);

        Ok(true)
    }
    fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Vec<u8>> {
        //remove from the msg to address to
        let from = self.vm().msg_sender();
        let bal = self.balance.get(from);
        assert_ne!(from, to, "cannot transfer to self");

        assert!(bal > U256::from(0), "insufficient balance");

        assert!(value > U256::from(0), "cannot send 0");

       Ok(self._transfer(from, bal, to, value))
    }
    fn mint(&mut self, to: Address, value: U256) {
        let bal = self.balance.get(to);
        self.balance.insert(to, bal+value);
        self.total_supply.set(self.total_supply.get()+value);
    }
}

impl ERC20 {
    fn _transfer(&mut self, from: Address, bal: U256,  to: Address, value: U256) -> bool {
        self.balance.insert(from, bal-value);
        self.balance.insert(to, self.balance.get(to)+value);
        true
    }
}

#[cfg(test)]
mod test {
    use alloy_primitives::U160;
    use super::*;
    use stylus_sdk::{testing::*, alloy_primitives::{U256, Address, }};

    fn create_erc20_instance() -> (ERC20, Address, TestVM) {
        let vm: TestVM = TestVM::default();
        let mut erc20 = ERC20::from(&vm);
        erc20.init();
        (erc20, vm.msg_sender(), vm)
    }

    #[test]
    fn test_get_decimals() {
        let (erc20, owner, _) = create_erc20_instance();
        assert_eq!(erc20.decimals(), 18);
        assert_eq!(erc20.name.get_string(), "Stylus");
        assert_eq!(erc20.owner.get(), owner);
        assert_eq!(erc20.total_supply.get(), U256::from(1_000_000_000));
        assert_eq!(erc20.balance.get(owner), U256::from(1_000_000_000));
    }

    #[test]
    // #[should_panic(expected = "[110, 111, 116, 32, 114, 101, 97, 100, 121]")]
    fn test_transfer() {
        let (mut erc20, owner, _) = create_erc20_instance();
        let to: Address = Address::from(U160::from(0x0000000000000000000000000000000000000001));

        erc20.transfer(to, U256::from(100)).unwrap();
        assert_eq!(erc20.balance.get(owner), U256::from(999_999_900));
        assert_eq!(erc20.balance.get(to), U256::from(100));
    }

    #[test]
    #[should_panic]
    fn test_transfer_error() {
        let (mut erc20, owner, vm) = create_erc20_instance();
        let to: Address = Address::from(U160::from(0x0000000000000000000000000000000000000001));
        // cannot transfer to self
        erc20.transfer(owner, U256::from(100)).unwrap();

        // cannot send 0
        // erc20.transfer(to, U256::from(0)).unwrap();
        assert_eq!(erc20.balance.get(owner), U256::from(1_000_000_000));

        //cannot transfer from 0 balance
        vm.set_sender(to);
        erc20.transfer(owner, U256::from(90)).unwrap();

    }

    #[test]
    fn test_transfer_from_and_approve() {
        let (mut erc20, owner, vm) = create_erc20_instance();
        let to: Address = Address::from(U160::from(0x0000000000000000000000000000000000000001));

        erc20.approve(to, U256::from(100)).unwrap();
        assert_eq!(erc20.allowance.get(owner).get(to), U256::from(100));

        vm.set_sender(to);
        erc20.transfer_from(owner, vm.contract_address(), U256::from(100)).unwrap();
        assert_eq!(erc20.balance.get(owner), U256::from(999_999_900));
        assert_eq!(erc20.balance.get(vm.contract_address()), U256::from(100));
        assert_eq!(erc20.balance.get(to), U256::from(0));
    }
}