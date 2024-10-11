pub mod prover;
pub mod srs;
pub mod utils;
pub mod traits;

use lambdaworks_math::{
    elliptic_curve::{
        short_weierstrass::curves::bls12_381::{
            curve::{
                BLS12381Curve,
                BLS12381FieldElement
            }, 
            traits::IsEllipticCurve,
        },
    },
    traits::ByteConversion,
    errors::ByteConversionError,
};

use icicle_bls12_381::curve;
use icicle_core::{error::IcicleError, msm, traits::FieldImpl};
use icicle_cuda_runtime::{ memory::{DeviceVec, HostSlice}, stream::CudaStream};


pub type G1Point = <BLS12381Curve as IsEllipticCurve>::PointRepresentation;

#[cfg(feature = "icicle")]
pub mod icicle;


pub type BlsG1point = ShortWeierstrassProjectivePoint<BLS12381Curve>;

// Trait to handle scalar conversion
pub trait ToIcicle {
    fn to_icicle_scalar(&self) -> curve::ScalarField;
    fn to_icicle(&self) -> curve::BaseField;
    fn from_icicle(icicle: &curve::BaseField) -> Result<Self, ByteConversionError>
    where
        Self: Sized;
}

pub impl ToIcicle for BLS12381FieldElement {
    fn to_icicle_scalar(&self) -> curve::ScalarField {
        let scalar_bytes = self.to_bytes_le();
        curve::ScalarField::from_bytes_le(&scalar_bytes)
    }

    fn to_icicle(&self) -> curve::BaseField {
        curve::BaseField::from_bytes_le(&self.to_bytes_le())
    }

    fn from_icicle(icicle: &curve::BaseField) -> Result<Self, ByteConversionError> {
        Self::from_bytes_le(&icicle.to_bytes_le())
    }
}

// Trait for point conversion to icicle format
pub trait PointConversion {
    fn to_icicle(&self) -> curve::G1Affine;
    fn from_icicle(icicle: &curve::G1Projective) -> Result<Self, ByteConversionError>
    where
        Self: Sized;
}

pub impl PointConversion for BlsG1point {
    fn to_icicle(&self) -> curve::G1Affine {
        let s = self.to_affine();
        let x = s.x().to_icicle();
        let y = s.y().to_icicle();
        curve::G1Affine { x, y }
    }

    fn from_icicle(icicle: &curve::G1Projective) -> Result<Self, ByteConversionError> {
        Ok(Self::new([
            ToIcicle::from_icicle(&icicle.x)?,
            ToIcicle::from_icicle(&icicle.y)?,
            ToIcicle::from_icicle(&icicle.z)?,
        ]))
    }
}

pub fn bls12_381_g1_msm(
    scalars:&[BLS12381FieldElement],
    points: &[BlsG1point],
    config: Option<msm::MSMConfig>,
) -> Result<BlsG1point, IcicleError> {
    let mut cfg = config.unwrap_or(msm::MSMConfig::default());

    let convert_scalars = scalars.iter()
            .map(|scalar| ToIcicle::to_icicle_scalar(scalar))
            .collect::<Vec<_>>();
    let icicle_scalars = HostSlice::from_slice(&convert_scalars);

    let convert_points = points
        .iter()
        .map(|point| PointConversion::to_icicle(point))
        .collect::<Vec<_>>();

    let icicle_points = HostSlice::from_slice(&convert_points);

    let mut msm_results = DeviceVec::<curve::G1Projective>::cuda_malloc(1).unwrap();
    let stream = CudaStream::create().unwrap();
    cfg.ctx.stream = &stream;
    cfg.is_async = true;
    msm::msm(icicle_scalars, icicle_points, &cfg, &mut msm_results[..]).unwrap();

    let mut msm_host_result = vec![curve::G1Projective::zero(); 1];

    stream.synchronize().unwrap();
    msm_results.copy_to_host(HostSlice::from_mut_slice(&mut msm_host_result[..])).unwrap();

    stream.destroy().unwrap();
    let res = PointConversion::from_icicle(&msm_host_result[0]).unwrap();
    Ok(res)
}