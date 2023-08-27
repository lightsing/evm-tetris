use primitive_types::U256;
use std::cmp::Ordering;

pub struct SignWrapper<T>(pub T);

pub trait SignExt: Sized + Copy + Ord {
    fn is_neg(&self) -> bool;
    fn neg(&self) -> Self;

    fn signed_cmp(&self, other: &Self) -> Ordering {
        use std::cmp::Ordering::*;

        match (self.is_neg(), other.is_neg()) {
            (true, false) => Less,
            (false, true) => Greater,
            (false, false) => self.cmp(other),
            (true, true) => other.cmp(self), // Reverse for negative numbers
        }
    }

    fn abs(&self) -> Self {
        if self.is_neg() {
            self.neg()
        } else {
            *self
        }
    }

    fn sign_wrap(self) -> SignWrapper<Self> {
        SignWrapper(self)
    }
}

impl<T: SignExt> Copy for SignWrapper<T> {}

impl<T: SignExt> Clone for SignWrapper<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: SignExt> Ord for SignWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.signed_cmp(&other.0)
    }
}

impl<T: SignExt> Eq for SignWrapper<T> {}

impl<T: SignExt> PartialEq<Self> for SignWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T: SignExt> PartialOrd<Self> for SignWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: SignExt> SignWrapper<T> {
    pub fn is_neg(&self) -> bool {
        self.0.is_neg()
    }

    pub fn neg(&self) -> Self {
        SignWrapper(self.0.neg())
    }
}

impl SignExt for U256 {
    fn is_neg(&self) -> bool {
        127 < self.byte(31)
    }

    fn neg(&self) -> Self {
        if self.is_zero() {
            *self
        } else {
            U256::from_big_endian(&[255u8; 32]) - *self + U256::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_is_neg() {
        assert_eq!(U256::from(0).is_neg(), false);
        assert_eq!(U256::from(1).is_neg(), false);
        assert_eq!(U256::from(127).is_neg(), false);
        assert_eq!(U256::from(128).is_neg(), false);
        assert_eq!(U256::from(255).is_neg(), false);
        assert_eq!(U256::from(256).is_neg(), false);
        assert_eq!(
            U256::from_str("0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap()
                .is_neg(),
            false
        );
        assert_eq!(
            U256::from_str("0x8000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .is_neg(),
            true
        );
        assert_eq!(
            U256::from_str("0x8000000000000000000000000000000000000000000000000000000000000001")
                .unwrap()
                .is_neg(),
            true
        );
        assert_eq!(
            U256::from_str("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap()
                .is_neg(),
            true
        );
    }

    #[test]
    fn test_signed_cmp() {
        let neg_big =
            U256::from_str("0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe")
                .unwrap();
        let neg_small =
            U256::from_str("0x8000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        let zero = U256::from(0);
        let pos_small = U256::from(1);
        let pos_big =
            U256::from_str("0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap();

        assert_eq!(zero.signed_cmp(&pos_small), std::cmp::Ordering::Less);
        assert_eq!(pos_small.signed_cmp(&zero), std::cmp::Ordering::Greater);
        assert_eq!(neg_small.signed_cmp(&zero), std::cmp::Ordering::Less);
        assert_eq!(zero.signed_cmp(&neg_small), std::cmp::Ordering::Greater);
        assert_eq!(neg_big.signed_cmp(&neg_small), std::cmp::Ordering::Less);
        assert_eq!(pos_big.signed_cmp(&pos_small), std::cmp::Ordering::Greater);
    }
}
