mod access_list;
mod bytecode;
mod gas;
mod memory;
mod opcodes;
mod stack;
mod storage;
mod utils;

use crate::evm::utils::SignExt;
pub use access_list::AccessList;
pub use bytecode::{Bytecode, Instruction};
pub use gas::{Gas, GasCost};
pub use memory::Memory;
pub use opcodes::OpcodeId;
use primitive_types::U256;
pub use stack::Stack;
pub use storage::Storage;

/// a simple emulator for the EVM
pub struct Evm {
    pub program_counter: usize,
    pub access_list: AccessList,
    pub bytecode: Bytecode,
    pub gas: Gas,
    pub memory: Memory,
    pub stack: Stack,
    pub storage: Storage,
}

#[derive(Debug, Clone)]
pub enum EvmError {
    OutOfGas,
    StackUnderflow,
    StackOverflow,
}

impl Evm {
    pub fn new(gas_limit: impl Into<GasCost>) -> Self {
        Evm {
            program_counter: 0,
            access_list: AccessList::default(),
            bytecode: Bytecode::default(),
            gas: Gas::new(gas_limit),
            memory: Memory::default(),
            stack: Stack::default(),
            storage: Storage::default(),
        }
    }

    pub fn into_parts(self) -> (usize, Bytecode, Stack, Memory, Gas, Storage, AccessList) {
        (
            self.program_counter,
            self.bytecode,
            self.stack,
            self.memory,
            self.gas,
            self.storage,
            self.access_list,
        )
    }

    pub fn from_parts(
        program_counter: usize,
        bytecode: Bytecode,
        stack: Stack,
        memory: Memory,
        gas: Gas,
        storage: Storage,
        access_list: AccessList,
    ) -> Self {
        Evm {
            program_counter,
            access_list,
            bytecode,
            gas,
            memory,
            stack,
            storage,
        }
    }

    pub fn push_instruction(&mut self, instruction: Instruction) -> Result<(), EvmError> {
        self.bytecode.push(instruction);
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), EvmError> {
        let opcode = self.bytecode.get_opcode(self.program_counter).unwrap();
        match opcode {
            OpcodeId::ADD | OpcodeId::MUL | OpcodeId::SUB => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                let a = self.stack.try_pop()?;
                let b = self.stack.try_pop()?;
                match opcode {
                    OpcodeId::ADD => self.stack.try_push(a.overflowing_add(b).0)?,
                    OpcodeId::MUL => self.stack.try_push(a.overflowing_mul(b).0)?,
                    OpcodeId::SUB => self.stack.try_push(a.overflowing_sub(b).0)?,
                    _ => unreachable!(),
                }
                self.program_counter += 1;
            }
            OpcodeId::DIV | OpcodeId::MOD => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                let a = self.stack.try_pop()?;
                let b = self.stack.try_pop()?;
                if b == U256::zero() {
                    self.stack.try_push(U256::zero())?;
                } else {
                    match opcode {
                        OpcodeId::DIV => self.stack.try_push(a / b)?,
                        OpcodeId::MOD => self.stack.try_push(a % b)?,
                        _ => unreachable!(),
                    }
                }
                self.program_counter += 1;
            }
            OpcodeId::SDIV | OpcodeId::SMOD => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                let a = self.stack.try_pop()?;
                let b = self.stack.try_pop()?;
                if b == U256::zero() {
                    self.stack.try_push(U256::zero())?;
                } else {
                    let a_sign = a.is_neg();
                    let b_sign = b.is_neg();
                    let result_sign = a_sign ^ b_sign;
                    let a_abs = if a_sign { a.neg() } else { a };
                    let b_abs = if b_sign { b.neg() } else { b };
                    let result_abs = match opcode {
                        OpcodeId::SDIV => a_abs / b_abs,
                        OpcodeId::SMOD => a_abs % b_abs,
                        _ => unreachable!(),
                    };
                    let result = if result_sign {
                        result_abs.neg()
                    } else {
                        result_abs
                    };
                    self.stack.try_push(result)?;
                }
                self.program_counter += 1;
            }
            OpcodeId::ADDMOD | OpcodeId::MULMOD => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                let a = self.stack.try_pop()?;
                let b = self.stack.try_pop()?;
                let n = self.stack.try_pop()?;
                if n == U256::zero() {
                    self.stack.try_push(U256::zero())?;
                } else {
                    let tmp = if opcode == OpcodeId::ADDMOD {
                        a.overflowing_add(b).0
                    } else {
                        a.overflowing_mul(b).0
                    };
                    self.stack.try_push(tmp % n)?;
                }
                self.program_counter += 1;
            }
            OpcodeId::EXP => unimplemented!(),
            OpcodeId::SIGNEXTEND => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                let least_significant_byte = self.stack.try_pop()?.low_u64() as usize;
                if least_significant_byte < 32 {
                    let number = self.stack.try_pop()?;
                    let byte_position = 8 * (31 - least_significant_byte);
                    let test_bit = U256::one() << byte_position;
                    let extend = if number & test_bit == U256::zero() {
                        U256::zero()
                    } else {
                        (!U256::zero()) << byte_position
                    };
                    let result = number | extend;
                    self.stack.try_push(result)?;
                } else {
                    // If the byte number is greater than 31, then push the number back without modification.
                    let number = self.stack.try_pop()?;
                    self.stack.try_push(number)?;
                }
                self.program_counter += 1;
            }
            OpcodeId::LT | OpcodeId::GT | OpcodeId::SLT | OpcodeId::SGT | OpcodeId::EQ => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                let a = self.stack.try_pop()?;
                let b = self.stack.try_pop()?;
                let condition = match opcode {
                    OpcodeId::LT => a < b,
                    OpcodeId::GT => a > b,
                    OpcodeId::SLT => a.sign_wrap() < b.sign_wrap(),
                    OpcodeId::SGT => a.sign_wrap() > b.sign_wrap(),
                    OpcodeId::EQ => a == b,
                    _ => unreachable!(),
                };
                if condition {
                    self.stack.try_push(U256::one())?;
                } else {
                    self.stack.try_push(U256::zero())?;
                }
                self.program_counter += 1;
            }
            OpcodeId::AND | OpcodeId::OR | OpcodeId::XOR => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                let a = self.stack.try_pop()?;
                let b = self.stack.try_pop()?;
                let result = match opcode {
                    OpcodeId::AND => a & b,
                    OpcodeId::OR => a | b,
                    OpcodeId::XOR => a ^ b,
                    _ => unreachable!(),
                };
                self.stack.try_push(result)?;
                self.program_counter += 1;
            }
            OpcodeId::NOT => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                let a = self.stack.try_pop()?;
                self.stack.try_push(!a)?;
                self.program_counter += 1;
            }
            OpcodeId::BYTE => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                let n = self.stack.try_pop()?;
                let a = self.stack.try_pop()?;
                let result = if n < U256::from(32) {
                    let byte_position = 8 * (31 - n.low_u64() as usize);
                    let test_bit = U256::one() << byte_position;
                    if a & test_bit == U256::zero() {
                        U256::zero()
                    } else {
                        U256::one()
                    }
                } else {
                    U256::zero()
                };
                self.stack.try_push(result)?;
                self.program_counter += 1;
            }
            OpcodeId::MLOAD => {
                self.memory.mload(&mut self.gas, &mut self.stack)?;
                self.program_counter += 1;
            }
            OpcodeId::MSTORE => {
                self.memory.mstore(&mut self.gas, &mut self.stack)?;
                self.program_counter += 1;
            }
            OpcodeId::MSTORE8 => {
                self.memory.mstore8(&mut self.gas, &mut self.stack)?;
                self.program_counter += 1;
            }
            OpcodeId::PC => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                self.stack.try_push(U256::from(self.program_counter))?;
                self.program_counter += 1;
            }
            OpcodeId::MSIZE => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                self.stack
                    .try_push(U256::from(self.memory.word_size() * 32))?;
                self.program_counter += 1;
            }
            OpcodeId::PUSH0 => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                self.stack.try_push(U256::zero())?;
                self.program_counter += 1;
            }
            _ if opcode.is_push() => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                let n_bytes = opcode.as_usize() - OpcodeId::PUSH0.as_usize();
                // left pad big endian bytes
                let mut bytes = [0u8; 32];
                for (idx, value) in self.bytecode.inner
                    [self.program_counter + 1..self.program_counter + 1 + n_bytes]
                    .iter()
                    .rev()
                    .enumerate()
                {
                    bytes[31 - idx] = value.value;
                }
                self.stack.try_push(U256::from_big_endian(&bytes))?;
                self.program_counter += 1 + n_bytes;
            }
            _ if opcode.is_dup() => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                self.stack
                    .try_dup(opcode.as_usize() - OpcodeId::DUP1.as_usize())?;
                self.program_counter += 1;
            }
            _ if opcode.is_swap() => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                self.stack
                    .try_swap(opcode.as_usize() - OpcodeId::SWAP1.as_usize())?;
                self.program_counter += 1;
            }
            OpcodeId::SHA3 => unimplemented!(),
            OpcodeId::SLOAD => {
                self.storage
                    .sload(&mut self.access_list, &mut self.gas, &mut self.stack)?;
                self.program_counter += 1;
            }
            OpcodeId::SSTORE => {
                self.storage
                    .sstore(&mut self.access_list, &mut self.gas, &mut self.stack)?;
                self.program_counter += 1;
            }
            OpcodeId::GAS => {
                self.gas.use_gas(opcode.constant_gas_cost())?;
                self.stack.try_push(U256::from(self.gas.left().as_u64()))?;
                self.program_counter += 1;
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::SmallVec;

    #[test]
    fn test_evm() {
        let mut evm = Evm::new(1000);
        evm.push_instruction(Instruction {
            opcode: OpcodeId::PUSH1,
            push_data: Some(SmallVec::from(vec![0x01])),
        })
        .unwrap();
        evm.push_instruction(Instruction {
            opcode: OpcodeId::PUSH1,
            push_data: Some(SmallVec::from(vec![0x02])),
        })
        .unwrap();
        evm.push_instruction(Instruction {
            opcode: OpcodeId::ADD,
            push_data: None,
        })
        .unwrap();
        evm.step().unwrap();
        evm.step().unwrap();
        evm.step().unwrap();
        assert_eq!(evm.stack.try_pop().unwrap(), U256::from(3));
    }
}
