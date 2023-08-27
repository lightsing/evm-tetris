use crate::evm::gas::GasCost;
use crate::evm::{EvmError, Gas, OpcodeId, Stack};
use primitive_types::U256;

/// Maximum size in bytes of memory
/// current is 512KB
pub const MAX_MEMORY_SIZE: usize = 512 * 1024;

#[derive(Debug, Clone)]
pub struct Memory {
    inner: Vec<u8>,
    word_size: usize,
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    /// create a new memory
    pub fn new() -> Self {
        Memory {
            inner: Vec::new(),
            word_size: 0,
        }
    }

    /// get the current word size
    /// could be used by `MSIZE` opcode
    pub fn word_size(&self) -> usize {
        self.word_size
    }

    /// Calculate the memory gas cost of the given byte size
    pub fn gas_cost(word_size: usize) -> GasCost {
        ((word_size * word_size) as u64 * GasCost::MEMORY_EXPANSION_QUAD_DENOMINATOR.as_u64()
            + word_size as u64 * GasCost::MEMORY_EXPANSION_LINEAR_COEFF.as_u64())
        .into()
    }

    /// try expand the memory to the given size
    /// could cause `OutOfGas` error
    ///
    /// # Returns
    ///
    /// Returns the new word size if success, or `OutOfGas` error.
    pub fn try_expand_to(
        &mut self,
        offset: U256,
        base_cost: GasCost,
        gas: &mut Gas,
    ) -> Result<usize, EvmError> {
        let offset: usize = offset.try_into().map_err(|_| EvmError::OutOfGas)?;
        if offset < self.word_size * 32 {
            gas.use_gas(base_cost)?;
            // no need to expand
            return Ok(self.word_size);
        }
        let prev_gas_cost = Self::gas_cost(self.word_size);
        let new_word_size = (offset + 31) / 32;
        let memory_expansion_cost = Self::gas_cost(new_word_size) - prev_gas_cost;
        gas.use_gas(base_cost + memory_expansion_cost)?;
        assert!(new_word_size * 32 <= MAX_MEMORY_SIZE);
        self.inner.resize(new_word_size * 32, 0);
        self.word_size = new_word_size;
        Ok(new_word_size)
    }

    /// Get a word from given offset
    ///
    /// # Stack Inputs
    /// - `offset`: the offset of the word
    ///
    /// # Stack Outputs
    /// - `value`: the value of the word
    ///
    /// # Panics
    ///
    /// Panics if the offset is out of range.
    pub fn raw_get(&self, offset: usize) -> U256 {
        U256::from_big_endian(&self.inner[offset..offset + 32])
    }

    /// Implementation of the MLOAD opcode
    ///
    /// # Stack Inputs
    ///
    /// - `offset`: the offset of the word
    ///
    /// # Stack Outputs
    ///
    /// - `value`: the value of the word
    ///
    /// # Returns
    ///
    /// Returns the value of the at the offset, or zero if the slot is empty.
    /// When gas is not enough, returns `OutOfGas` error.
    /// When the stack is empty, returns `StackUnderflow` error.
    pub fn mload(&mut self, gas: &mut Gas, stack: &mut Stack) -> Result<U256, EvmError> {
        let static_gas = OpcodeId::MLOAD.constant_gas_cost();

        // key: memory offset to be read.
        let offset = stack.try_pop()?;
        self.try_expand_to(offset, static_gas, gas)?;

        let value = self.raw_get(offset.as_usize());
        stack.try_push(value).unwrap(); // impossible to fail, so unwrap
        Ok(value)
    }

    /// Implementation of the MSTORE opcode
    ///
    /// # Stack Inputs
    /// - `offset`: the offset of the word
    /// - `value`: the value of the word
    ///
    /// # Returns
    ///
    /// Returns nothing.
    /// When gas is not enough, returns `OutOfGas` error.
    /// When the stack has less than 2 elements, returns `StackUnderflow` error.
    pub fn mstore(&mut self, gas: &mut Gas, stack: &mut Stack) -> Result<(), EvmError> {
        let static_gas = OpcodeId::MSTORE.constant_gas_cost();

        // offset: memory offset to be modified.
        let offset = stack.try_pop()?;
        // value: value to be stored in the memory.
        let value = stack.try_pop()?;
        self.try_expand_to(offset, static_gas, gas)?;

        let offset = offset.as_usize();
        let value = value.to_big_endian(&mut self.inner[offset..offset + 32]);
        Ok(())
    }

    /// Implementation of the MSTORE8 opcode
    ///
    /// # Stack Inputs
    /// - `offset`: the offset of the byte
    /// - `value`: 1-byte value to write in the memory (the least significant byte of the 32-byte stack value).
    ///
    /// # Returns
    ///
    /// Returns nothing.
    /// When gas is not enough, returns `OutOfGas` error.
    /// When the stack has less than 2 elements, returns `StackUnderflow` error.
    pub fn mstore8(&mut self, gas: &mut Gas, stack: &mut Stack) -> Result<(), EvmError> {
        let static_gas = OpcodeId::MSTORE8.constant_gas_cost();

        // offset: memory offset to be modified.
        let offset = stack.try_pop()?;
        // value: value to be stored in the memory.
        let value = stack.try_pop()?;
        self.try_expand_to(offset, static_gas, gas)?;

        let offset = offset.as_usize();
        self.inner[offset] = value.byte(0);
        Ok(())
    }
}
