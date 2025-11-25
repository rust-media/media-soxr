use std::ffi::{c_uint, c_ulong};

use bitflags::bitflags;

use crate::{data_type::DataType, error::Result, soxr_sys};

#[derive(Clone, Debug)]
pub struct IOSpec {
    pub(crate) io_spec: soxr_sys::soxr_io_spec,
}

impl IOSpec {
    pub fn new(input_type: DataType, output_type: DataType) -> Result<Self> {
        let spec = unsafe { soxr_sys::soxr_io_spec(input_type.try_into()?, output_type.try_into()?) };

        if !spec.e.is_null() {
            return Err(crate::error::Error::new(spec.e as soxr_sys::soxr_error_t));
        }

        Ok(Self {
            io_spec: spec,
        })
    }

    pub fn input_type(&self) -> DataType {
        self.io_spec.itype.into()
    }

    pub fn output_type(&self) -> DataType {
        self.io_spec.otype.into()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum QualityRecipe {
    Quick    = soxr_sys::SOXR_QQ,
    Low      = soxr_sys::SOXR_LQ,
    Medium   = soxr_sys::SOXR_MQ,
    High     = soxr_sys::SOXR_20_BITQ,
    VeryHigh = soxr_sys::SOXR_28_BITQ,
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct QualityFlags: u32 {
        const RolloffSmall = soxr_sys::SOXR_ROLLOFF_SMALL;
        const RolloffMedium = soxr_sys::SOXR_ROLLOFF_MEDIUM;
        const RolloffNone = soxr_sys::SOXR_ROLLOFF_NONE;
        const HiPrecClock = soxr_sys::SOXR_HI_PREC_CLOCK;
        const DoublePrecision = soxr_sys::SOXR_DOUBLE_PRECISION;
        const VR = soxr_sys::SOXR_VR;
    }
}

pub struct QualitySpec {
    pub(crate) quality_spec: soxr_sys::soxr_quality_spec,
}

impl QualitySpec {
    pub fn new(recipe: QualityRecipe, flags: QualityFlags) -> Result<Self> {
        let spec = unsafe { soxr_sys::soxr_quality_spec(recipe as c_ulong, flags.bits() as c_ulong) };

        if !spec.e.is_null() {
            return Err(crate::error::Error::new(spec.e as soxr_sys::soxr_error_t));
        }

        Ok(Self {
            quality_spec: spec,
        })
    }
}

pub struct RuntimeSpec {
    pub(crate) runtime_spec: soxr_sys::soxr_runtime_spec,
}

impl RuntimeSpec {
    pub fn new(num_threads: u32) -> Self {
        Self {
            runtime_spec: unsafe { soxr_sys::soxr_runtime_spec(num_threads as c_uint) },
        }
    }
}
