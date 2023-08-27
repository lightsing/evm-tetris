use crate::evm::OpcodeId;
use rand::prelude::*;
use std::fmt::Display;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Bytecode {
    pub inner: Vec<BytecodeElement>,
}

/// Helper struct that represents a single element in a bytecode.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct BytecodeElement {
    /// The byte value of the element.
    pub value: u8,
    /// Whether the element is an opcode or push data byte.
    pub is_code: bool,
}

impl Bytecode {
    pub fn get_opcode(&self, index: usize) -> Option<OpcodeId> {
        let element = self.inner[index];
        if element.is_code {
            Some(OpcodeId::from(element.value))
        } else {
            None
        }
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.inner.push(BytecodeElement {
            value: instruction.opcode.as_u8(),
            is_code: true,
        });
        if let Some(push_data) = instruction.push_data {
            for byte in push_data {
                self.inner.push(BytecodeElement {
                    value: byte,
                    is_code: false,
                });
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: OpcodeId,
    pub push_data: Option<[u8; 32]>,
}

impl Instruction {
    pub fn random(mut rng: impl RngCore) -> Self {
        let opcode = OpcodeId::random(&mut rng);
        if opcode.is_push() && opcode != OpcodeId::PUSH0 {
            let n_bytes = opcode.as_usize() - OpcodeId::PUSH0.as_usize();
            let mut push_data = [0u8; 32];
            rng.fill(&mut push_data[0..n_bytes]);
            Self {
                opcode,
                push_data: Some(push_data),
            }
        } else {
            Self {
                opcode,
                push_data: None,
            }
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.opcode)?;
        if let Some(push_data) = &self.push_data {
            write!(
                f,
                " 0x{}",
                hex::encode(&push_data[0..(self.opcode.as_usize() - OpcodeId::PUSH0.as_usize())])
            )?;
        }
        Ok(())
    }
}
