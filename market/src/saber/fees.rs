//! Program fees

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{Pack, Sealed},
};
use rust_decimal::prelude::ToPrimitive;

/// Fees struct
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Fees {
    /// Admin trade fee numerator
    pub admin_trade_fee_numerator: u64,
    /// Admin trade fee denominator
    pub admin_trade_fee_denominator: u64,
    /// Admin withdraw fee numerator
    pub admin_withdraw_fee_numerator: u64,
    /// Admin withdraw fee denominator
    pub admin_withdraw_fee_denominator: u64,
    /// Trade fee numerator
    pub trade_fee_numerator: u64,
    /// Trade fee denominator
    pub trade_fee_denominator: u64,
    /// Withdraw fee numerator
    pub withdraw_fee_numerator: u64,
    /// Withdraw fee denominator
    pub withdraw_fee_denominator: u64,
}

impl Fees {
    /// Apply admin trade fee
    pub fn admin_trade_fee(&self, fee_amount: u64) -> Option<u64> {
        mul_div_imbalanced(
            fee_amount,
            self.admin_trade_fee_numerator,
            self.admin_trade_fee_denominator,
        )
    }

    /// Apply admin withdraw fee
    pub fn admin_withdraw_fee(&self, fee_amount: u64) -> Option<u64> {
        mul_div_imbalanced(
            fee_amount,
            self.admin_withdraw_fee_numerator,
            self.admin_withdraw_fee_denominator,
        )
    }

    /// Compute trade fee from amount
    pub fn trade_fee(&self, trade_amount: u64) -> Option<u64> {
        mul_div_imbalanced(
            trade_amount,
            self.trade_fee_numerator,
            self.trade_fee_denominator,
        )
    }

    /// Compute withdraw fee from amount
    pub fn withdraw_fee(&self, withdraw_amount: u64) -> Option<u64> {
        mul_div_imbalanced(
            withdraw_amount,
            self.withdraw_fee_numerator,
            self.withdraw_fee_denominator,
        )
    }

    /// Compute normalized fee for symmetric/asymmetric deposits/withdraws
    pub fn normalized_trade_fee(&self, n_coins: u8, amount: u64) -> Option<u64> {
        // adjusted_fee_numerator: uint256 = self.fee * N_COINS / (4 * (N_COINS - 1))
        // The number 4 comes from Curve, originating from some sort of calculus
        // https://github.com/curvefi/curve-contract/blob/e5fb8c0e0bcd2fe2e03634135806c0f36b245511/tests/simulation.py#L124
        let adjusted_trade_fee_numerator = mul_div(
            self.trade_fee_numerator,
            n_coins.into(),
            (n_coins.checked_sub(1)?).checked_mul(4)?.into(),
        )?;

        mul_div(
            amount,
            adjusted_trade_fee_numerator,
            self.trade_fee_denominator,
        )
    }
}

const MAX: u64 = 1 << 32;
const MAX_BIG: u64 = 1 << 48;
const MAX_SMALL: u64 = 1 << 16;

pub fn mul_div(a: u64, b: u64, c: u64) -> Option<u64> {
    if a > MAX || b > MAX {
        (a as u128)
            .checked_mul(b as u128)?
            .checked_div(c as u128)?
            .to_u64()
    } else {
        a.checked_mul(b)?.checked_div(c)
    }
}

pub fn mul_div_imbalanced(a: u64, b: u64, c: u64) -> Option<u64> {
    if a > MAX_BIG || b > MAX_SMALL {
        (a as u128)
            .checked_mul(b as u128)?
            .checked_div(c as u128)?
            .to_u64()
    } else {
        a.checked_mul(b)?.checked_div(c)
    }
}

impl Sealed for Fees {}
impl Pack for Fees {
    const LEN: usize = 64;
    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, 64];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            admin_trade_fee_numerator,
            admin_trade_fee_denominator,
            admin_withdraw_fee_numerator,
            admin_withdraw_fee_denominator,
            trade_fee_numerator,
            trade_fee_denominator,
            withdraw_fee_numerator,
            withdraw_fee_denominator,
        ) = array_refs![input, 8, 8, 8, 8, 8, 8, 8, 8];
        Ok(Self {
            admin_trade_fee_numerator: u64::from_le_bytes(*admin_trade_fee_numerator),
            admin_trade_fee_denominator: u64::from_le_bytes(*admin_trade_fee_denominator),
            admin_withdraw_fee_numerator: u64::from_le_bytes(*admin_withdraw_fee_numerator),
            admin_withdraw_fee_denominator: u64::from_le_bytes(*admin_withdraw_fee_denominator),
            trade_fee_numerator: u64::from_le_bytes(*trade_fee_numerator),
            trade_fee_denominator: u64::from_le_bytes(*trade_fee_denominator),
            withdraw_fee_numerator: u64::from_le_bytes(*withdraw_fee_numerator),
            withdraw_fee_denominator: u64::from_le_bytes(*withdraw_fee_denominator),
        })
    }

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, 64];
        let (
            admin_trade_fee_numerator,
            admin_trade_fee_denominator,
            admin_withdraw_fee_numerator,
            admin_withdraw_fee_denominator,
            trade_fee_numerator,
            trade_fee_denominator,
            withdraw_fee_numerator,
            withdraw_fee_denominator,
        ) = mut_array_refs![output, 8, 8, 8, 8, 8, 8, 8, 8];
        *admin_trade_fee_numerator = self.admin_trade_fee_numerator.to_le_bytes();
        *admin_trade_fee_denominator = self.admin_trade_fee_denominator.to_le_bytes();
        *admin_withdraw_fee_numerator = self.admin_withdraw_fee_numerator.to_le_bytes();
        *admin_withdraw_fee_denominator = self.admin_withdraw_fee_denominator.to_le_bytes();
        *trade_fee_numerator = self.trade_fee_numerator.to_le_bytes();
        *trade_fee_denominator = self.trade_fee_denominator.to_le_bytes();
        *withdraw_fee_numerator = self.withdraw_fee_numerator.to_le_bytes();
        *withdraw_fee_denominator = self.withdraw_fee_denominator.to_le_bytes();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn pack_fees() {
        let admin_trade_fee_numerator = 1;
        let admin_trade_fee_denominator = 2;
        let admin_withdraw_fee_numerator = 3;
        let admin_withdraw_fee_denominator = 4;
        let trade_fee_numerator = 5;
        let trade_fee_denominator = 6;
        let withdraw_fee_numerator = 7;
        let withdraw_fee_denominator = 8;
        let fees = Fees {
            admin_trade_fee_numerator,
            admin_trade_fee_denominator,
            admin_withdraw_fee_numerator,
            admin_withdraw_fee_denominator,
            trade_fee_numerator,
            trade_fee_denominator,
            withdraw_fee_numerator,
            withdraw_fee_denominator,
        };

        let mut packed = [0u8; Fees::LEN];
        Pack::pack_into_slice(&fees, &mut packed[..]);
        let unpacked = Fees::unpack_from_slice(&packed).unwrap();
        assert_eq!(fees, unpacked);

        let mut packed = vec![];
        packed.extend_from_slice(&admin_trade_fee_numerator.to_le_bytes());
        packed.extend_from_slice(&admin_trade_fee_denominator.to_le_bytes());
        packed.extend_from_slice(&admin_withdraw_fee_numerator.to_le_bytes());
        packed.extend_from_slice(&admin_withdraw_fee_denominator.to_le_bytes());
        packed.extend_from_slice(&trade_fee_numerator.to_le_bytes());
        packed.extend_from_slice(&trade_fee_denominator.to_le_bytes());
        packed.extend_from_slice(&withdraw_fee_numerator.to_le_bytes());
        packed.extend_from_slice(&withdraw_fee_denominator.to_le_bytes());
        let unpacked = Fees::unpack_from_slice(&packed).unwrap();
        assert_eq!(fees, unpacked);
    }
}
