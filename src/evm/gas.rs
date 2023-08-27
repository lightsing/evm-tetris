use crate::evm::EvmError;
use std::ops::{Add, AddAssign, Sub};

/// Defines the gas consumption.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct GasCost(u64);

impl GasCost {
    /// Constant cost for free step
    pub const ZERO: Self = Self(0);
    // /// Constant cost for jumpdest step, only takes one gas
    // pub const ONE: Self = Self(1);
    /// Constant cost for quick step
    pub const QUICK: Self = Self(2);
    /// Constant cost for fastest step
    pub const FASTEST: Self = Self(3);
    /// Constant cost for fast step
    pub const FAST: Self = Self(5);
    /// Constant cost for mid step
    pub const MID: Self = Self(8);
    /// Constant cost for slow step
    pub const SLOW: Self = Self(10);
    // /// Constant cost for ext step
    // pub const EXT: Self = Self(20);
    /// Constant cost for SHA3
    pub const SHA3: Self = Self(30);
    // /// Constant cost for SELFDESTRUCT
    // pub const SELFDESTRUCT: Self = Self(5000);
    // /// Constant cost for CREATE and CREATE2
    // pub const CREATE: Self = Self(32000);
    /// Constant cost for copying every word
    pub const COPY: Self = Self(3);
    /// Constant cost for copying every word, specifically in the case of SHA3
    /// opcode.
    pub const COPY_SHA3: Self = Self(6);
    /// Constant cost for accessing account or storage key
    pub const WARM_ACCESS: Self = Self(100);
    /// Constant cost for a cold SLOAD
    pub const COLD_SLOAD: Self = Self(2100);
    // /// Constant cost for a cold account access
    // pub const COLD_ACCOUNT_ACCESS: Self = Self(2600);
    /// SSTORE reentrancy sentry
    pub const SSTORE_SENTRY: Self = Self(2300);
    /// Constant cost for a storage set
    pub const SSTORE_SET: Self = Self(20000);
    /// Constant cost for a storage reset
    pub const SSTORE_RESET: Self = Self(2900);
    /// Constant cost for a storage clear. EIP-3529 changed it to 4800 from
    /// 15000.
    pub const SSTORE_CLEARS_SCHEDULE: Self = Self(4800);
    // /// Constant cost for a non-creation transaction
    // pub const TX: Self = Self(21000);
    // /// Constant cost for a creation transaction
    // pub const CREATION_TX: Self = Self(53000);
    // /// Constant cost for calling with non-zero value
    // pub const CALL_WITH_VALUE: Self = Self(9000);
    // /// Constant cost for turning empty account into non-empty account
    // pub const NEW_ACCOUNT: Self = Self(25000);
    // /// Cost per byte of deploying a new contract
    // pub const CODE_DEPOSIT_BYTE_COST: Self = Self(200);
    /// Denominator of quadratic part of memory expansion gas cost
    pub const MEMORY_EXPANSION_QUAD_DENOMINATOR: Self = Self(512);
    /// Coefficient of linear part of memory expansion gas cost
    pub const MEMORY_EXPANSION_LINEAR_COEFF: Self = Self(3);
    // /// Constant gas for LOG[0-4] op codes
    // pub const LOG: Self = Self(375);
    /// Times ceil exponent byte size for the EXP instruction, EIP-158 changed
    /// it from 10 to 50.
    pub const EXP_BYTE_TIMES: Self = Self(50);
    // /// Base gas cost for precompile call: Elliptic curve recover
    // pub const PRECOMPILE_ECRECOVER_BASE: Self = Self(3_000);
    // /// Base gas cost for precompile call: SHA256
    // pub const PRECOMPILE_SHA256_BASE: Self = Self(60);
    // /// Per-word gas cost for SHA256
    // pub const PRECOMPILE_SHA256_PER_WORD: Self = Self(12);
    // /// Base gas cost for precompile call: RIPEMD160
    // pub const PRECOMPILE_RIPEMD160_BASE: Self = Self(600);
    // /// Per-word gas cost for RIPEMD160
    // pub const PRECOMPILE_RIPEMD160_PER_WORD: Self = Self(120);
    // /// Base gas cost for precompile call: Identity
    // pub const PRECOMPILE_IDENTITY_BASE: Self = Self(15);
    // /// Per-word gas cost for Identity
    // pub const PRECOMPILE_IDENTITY_PER_WORD: Self = Self(3);
    // /// Base gas cost for precompile call: BN256 point addition
    // pub const PRECOMPILE_BN256ADD: Self = Self(150);
    // /// Base gas cost for precompile call: BN256 scalar multiplication
    // pub const PRECOMPILE_BN256MUL: Self = Self(6_000);
    // /// Base gas cost for precompile call: BN256 pairing op base cost
    // pub const PRECOMPILE_BN256PAIRING: Self = Self(45_000);
    // /// Per-pair gas cost for BN256 pairing
    // pub const PRECOMPILE_BN256PAIRING_PER_PAIR: Self = Self(34_000);
    // /// Base gas cost for precompile call: MODEXP
    // pub const PRECOMPILE_MODEXP: Self = Self(0);
    // /// Minimum gas cost for precompile calls: MODEXP
    // pub const PRECOMPILE_MODEXP_MIN: Self = Self(200);
    // /// Base gas cost for precompile call: BLAKE2F
    // pub const PRECOMPILE_BLAKE2F: Self = Self(0);

    /// Returns the gas cost.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl From<u64> for GasCost {
    fn from(value: u64) -> Self {
        GasCost(value)
    }
}

impl Add<GasCost> for GasCost {
    type Output = GasCost;

    fn add(self, rhs: GasCost) -> Self::Output {
        GasCost(self.0 + rhs.0)
    }
}

impl Add<u64> for GasCost {
    type Output = GasCost;

    fn add(self, rhs: u64) -> Self::Output {
        GasCost(self.0 + rhs)
    }
}

impl AddAssign<GasCost> for GasCost {
    fn add_assign(&mut self, rhs: GasCost) {
        self.0 += rhs.0;
    }
}

impl AddAssign<u64> for GasCost {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl Sub<GasCost> for GasCost {
    type Output = GasCost;

    fn sub(self, rhs: GasCost) -> Self::Output {
        GasCost(self.0 - rhs.0)
    }
}

impl Sub<u64> for GasCost {
    type Output = GasCost;

    fn sub(self, rhs: u64) -> Self::Output {
        GasCost(self.0 - rhs)
    }
}

/// Gas Manager for EVM
#[derive(Debug, Clone, Copy)]
pub struct Gas {
    /// Gas limit
    limit: GasCost,
    /// Gas used
    used: GasCost,
}

impl Gas {
    /// Create a new gas manager
    pub fn new(limit: impl Into<GasCost>) -> Self {
        Gas {
            limit: limit.into(),
            used: 0.into(),
        }
    }

    /// Returns the gas limit
    pub fn limit(&self) -> GasCost {
        self.limit
    }

    /// Returns the gas used
    pub fn used(&self) -> GasCost {
        self.used
    }

    /// Returns the gas left
    pub fn left(&self) -> GasCost {
        self.limit - self.used
    }

    /// Returns Ok if the gas is enough
    pub fn enough(&self, cost: impl Into<GasCost>) -> Result<(), EvmError> {
        let cost = cost.into();
        if self.left() >= cost {
            Ok(())
        } else {
            Err(EvmError::OutOfGas)
        }
    }

    /// Use the gas
    pub fn use_gas(&mut self, cost: GasCost) -> Result<(), EvmError> {
        if let Err(e) = self.enough(cost) {
            return Err(e);
        }
        self.used += cost.as_u64();
        Ok(())
    }
}
