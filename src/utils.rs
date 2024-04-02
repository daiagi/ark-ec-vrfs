use ark_ff::PrimeField;

use crate::{AffinePoint, Suite};

#[macro_export]
macro_rules! suite_types {
    ($suite:ident) => {
        #[allow(dead_code)]
        pub type Secret = crate::Secret<$suite>;
        #[allow(dead_code)]
        pub type Public = crate::Public<$suite>;
        #[allow(dead_code)]
        pub type Input = crate::Input<$suite>;
        #[allow(dead_code)]
        pub type Output = crate::Output<$suite>;
        #[allow(dead_code)]
        pub type AffinePoint = crate::AffinePoint<$suite>;
        #[allow(dead_code)]
        pub type ScalarField = crate::ScalarField<$suite>;
    };
}

/// SHA-512 hasher
#[inline(always)]
pub(crate) fn sha512(input: &[u8]) -> [u8; 64] {
    use sha2::{Digest, Sha512};
    let mut hasher = Sha512::new();
    hasher.update(input);
    let result = hasher.finalize();
    let mut h = [0u8; 64];
    h.copy_from_slice(&result);
    h
}

/// Blake2b
#[inline(always)]
pub(crate) fn blake2(input: &[u8]) -> [u8; 64] {
    use blake2b_simd::blake2b;
    *blake2b(input).as_array()
}

/// Try-And-Increment (TAI) method as defined by RFC9381 section 5.4.1.1.
///
/// Implements ECVRF_encode_to_curve in a simple and generic way that works
/// for any elliptic curve.
///
/// To use this algorithm, hash length MUST be at least equal to the field length.
pub fn hash_to_curve_tai<S: Suite>(data: &[u8]) -> Option<AffinePoint<S>> {
    use ark_ec::AffineRepr;
    use ark_ff::Field;
    use ark_serialize::CanonicalDeserialize;

    const DOM_SEP_FRONT: u8 = 0x01;
    const DOM_SEP_BACK: u8 = 0x00;

    let mod_size = <<<S::Affine as AffineRepr>::BaseField as Field>::BasePrimeField as PrimeField>::MODULUS_BIT_SIZE as usize / 8;

    for ctr in 0..256 {
        let buf = [
            &[S::SUITE_ID, DOM_SEP_FRONT],
            data,
            &[ctr as u8, DOM_SEP_BACK],
        ]
        .concat();
        let hash = &S::hash(&buf)[..];
        if hash.len() < mod_size {
            return None;
        }
        if let Ok(pt) = AffinePoint::<S>::deserialize_compressed_unchecked(&hash[..]) {
            return Some(pt.clear_cofactor());
        }
    }
    None
}

#[cfg(test)]
pub(crate) mod testing {
    use crate::*;

    pub const TEST_SEED: &[u8] = b"test seed";

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct TestSuite;

    impl Suite for TestSuite {
        const SUITE_ID: u8 = 0xFF;
        const CHALLENGE_LEN: usize = 16;

        type Affine = ark_ed25519::EdwardsAffine;
        type Hash = [u8; 64];

        fn hash(data: &[u8]) -> Self::Hash {
            utils::sha512(data)
        }
    }

    suite_types!(TestSuite);
}

#[cfg(test)]
mod tests {
    use super::*;
    use testing::TestSuite;

    #[test]
    fn hash_to_curve_tai_works() {
        let pt = hash_to_curve_tai::<TestSuite>(b"hello world").unwrap();
        // Check that pt is in the prime subgroup
        assert!(pt.is_on_curve());
        assert!(pt.is_in_correct_subgroup_assuming_on_curve())
    }
}
