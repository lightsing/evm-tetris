use crate::evm::{AccessList, EvmError, Gas, GasCost, OpcodeId, Stack};
use primitive_types::U256;
use std::collections::HashMap;

pub struct Storage {
    inner: HashMap<U256, U256>,
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            inner: HashMap::new(),
        }
    }

    pub fn raw_get(&self, key: U256) -> U256 {
        self.inner.get(&key).copied().unwrap_or_default()
    }

    pub fn raw_set(&mut self, key: U256, value: U256) -> U256 {
        self.inner.insert(key, value).unwrap_or_default()
    }

    pub fn raw_clear(&mut self, key: U256) {
        self.inner.remove(&key);
    }

    /// Implementation of the SLOAD opcode
    ///
    /// # Returns
    ///
    /// Returns the value of the storage slot, or zero if the slot is empty.
    /// When gas is not enough, returns `OutOfGas` error.
    /// When the stack is empty, returns `StackUnderflow` error.
    pub fn sload(
        &mut self,
        access_list: &mut AccessList,
        gas: &mut Gas,
        stack: &mut Stack,
    ) -> Result<U256, EvmError> {
        let static_gas = OpcodeId::SLOAD.constant_gas_cost();

        // key: storage slot to be read.
        let key = stack.try_pop()?;

        let dynamic_gas = if access_list.is_warm_slot(key) {
            GasCost::WARM_ACCESS
        } else {
            GasCost::COLD_SLOAD
        };
        let total_gas = static_gas + dynamic_gas;
        gas.use_gas(total_gas)?;
        access_list.add_warm_slot(key);
        let value = self.raw_get(key);
        stack.try_push(value).unwrap(); // impossible to fail, so unwrap

        Ok(value)
    }

    /// Implementation of the SSTORE opcode
    ///
    /// # Returns
    ///
    /// Returns the original value of the storage slot, or zero if the slot was empty.
    /// When gas is not enough, returns `OutOfGas` error.
    /// When the stack has less than 2 elements, returns `StackUnderflow` error.
    pub fn sstore(
        &mut self,
        access_list: &mut AccessList,
        gas: &mut Gas,
        stack: &mut Stack,
    ) -> Result<U256, EvmError> {
        let static_gas = OpcodeId::SSTORE.constant_gas_cost();

        // key: storage slot to be modified.
        let key = stack.try_pop()?;
        // value: value to be stored in the storage slot.
        let value = stack.try_pop()?;

        // current value of the storage slot.
        let current_value = self.raw_get(key);
        // original_value: value of the storage slot before the current transaction.
        // in this game, original value is always zero
        // if value == current_value
        //     if key is warm
        //         base_dynamic_gas = GasCost::WARM_ACCESS
        //     else
        //         base_dynamic_gas = GasCost::WARM_ACCESS
        // else if current_value == original_value // aka. current_value == 0
        //     if original_value == 0       // always true
        //         base_dynamic_gas = GasCost::SSTORE_SET
        //     else
        //         base_dynamic_gas = GasCost::SSTORE_RESET
        // else
        //     base_dynamic_gas = GasCost::WARM_ACCESS
        let base_dynamic_gas = if current_value.is_zero() {
            GasCost::SSTORE_SET
        } else {
            GasCost::WARM_ACCESS
        };
        let dynamic_gas = if access_list.is_warm_slot(key) {
            base_dynamic_gas
        } else {
            base_dynamic_gas + GasCost::COLD_SLOAD
        };
        let total_gas = static_gas + dynamic_gas;
        gas.use_gas(total_gas)?;
        access_list.add_warm_slot(key);
        Ok(self.raw_set(key, value))
    }
}
