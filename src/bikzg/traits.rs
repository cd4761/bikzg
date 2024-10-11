use lambdaworks_math::field::element::FieldElement;
use lambdaworks_math::field::traits::IsField;
use lambdaworks_math::errors::{ByteConversionError, DeserializationError};

use crate::bipolynomial::BivariatePolynomial;
use lambdaworks_math::polynomial::Polynomial as UnivariatePolynomial;



pub trait IsCommitmentScheme<F: IsField> {
    type Commitment;


    fn commit_bivariate(&self, bp: &BivariatePolynomial<FieldElement<F>>) -> Self::Commitment;
    fn commit_univariate(&self, bp: &UnivariatePolynomial<FieldElement<F>>) -> Self::Commitment;
    fn icicle_commit_bivariate(&self, bp: &BivariatePolynomial<FieldElement<F>>) -> Self::Commitment;

    fn open(
        &self,
        x: &FieldElement<F>,
        y: &FieldElement<F>,
        evaluation: &FieldElement<F>,//f(x,y)
        p: &BivariatePolynomial<FieldElement<F>>,
    ) -> (Self::Commitment,Self::Commitment);


    fn verify(
        &self,
        x: &FieldElement<F>,
        y: &FieldElement<F>,
        evaluation: &FieldElement<F>,
        p_commitment: &Self::Commitment,
        proofs: &(Self::Commitment,Self::Commitment),
    ) -> bool;

}

pub trait ByteConversion {
    /// Returns the byte representation of the element in big-endian order.}
    #[cfg(feature = "alloc")]
    fn to_bytes_be(&self) -> alloc::vec::Vec<u8>;

    /// Returns the byte representation of the element in little-endian order.
    #[cfg(feature = "alloc")]
    fn to_bytes_le(&self) -> alloc::vec::Vec<u8>;

    /// Returns the element from its byte representation in big-endian order.
    fn from_bytes_be(bytes: &[u8]) -> Result<Self, ByteConversionError>
    where
        Self: Sized;

    /// Returns the element from its byte representation in little-endian order.
    fn from_bytes_le(bytes: &[u8]) -> Result<Self, ByteConversionError>
    where
        Self: Sized;
}

/// Serialize function without args
/// Used for serialization when formatting options are not relevant
#[cfg(feature = "alloc")]
pub trait AsBytes {
    /// Default serialize without args
    fn as_bytes(&self) -> alloc::vec::Vec<u8>;
}

#[cfg(feature = "alloc")]
impl AsBytes for u32 {
    fn as_bytes(&self) -> alloc::vec::Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

#[cfg(feature = "alloc")]
impl AsBytes for u64 {
    fn as_bytes(&self) -> alloc::vec::Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

/// Deserialize function without args
pub trait Deserializable {
    fn deserialize(bytes: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized;
}