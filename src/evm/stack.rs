use crate::evm::EvmError;
use primitive_types::U256;

pub const MAX_STACK_SIZE: usize = 1024;

/// a simple stack for the EVM
#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    pub inner: Vec<U256>,
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    /// create a new stack
    pub fn new() -> Self {
        Stack { inner: Vec::new() }
    }

    /// push a value onto the stack
    pub fn try_push(&mut self, value: U256) -> Result<(), EvmError> {
        if self.inner.len() >= MAX_STACK_SIZE {
            return Err(EvmError::StackOverflow);
        }
        self.inner.push(value);
        Ok(())
    }

    /// pop a value from the stack
    pub fn try_pop(&mut self) -> Result<U256, EvmError> {
        self.inner.pop().ok_or(EvmError::StackUnderflow)
    }

    /// get the length of the stack
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// check if the stack has at least n elements
    pub fn try_at_least(&self, n: usize) -> Result<(), EvmError> {
        if self.inner.len() >= n {
            Ok(())
        } else {
            Err(EvmError::StackUnderflow)
        }
    }

    /// Dup top n-th value on the stack
    /// used by DUP1 - DUP16, with n = 0..16
    pub fn try_dup(&mut self, n: usize) -> Result<(), EvmError> {
        self.try_at_least(n + 1)?;
        let len = self.inner.len();
        self.try_push(self.inner[len - n - 1])
    }

    /// swap the top two values on the stack
    /// used by SWAP1 - SWAP16, with n = 0..16
    pub fn try_swap(&mut self, n: usize) -> Result<(), EvmError> {
        self.try_at_least(n + 2)?;
        let len = self.inner.len();
        self.inner.swap(len - 1, len - 1 - n);
        Ok(())
    }
}
