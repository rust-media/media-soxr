use crate::{
    error::{Error, Result},
    soxr_sys::soxr_datatype_t,
};

const UNSUPPORTED_DATA_TYPE: &str = "unsupported data type";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum DataType {
    Float32I = soxr_datatype_t::SOXR_FLOAT32_I as i32,
    Float64I = soxr_datatype_t::SOXR_FLOAT64_I as i32,
    Int32I   = soxr_datatype_t::SOXR_INT32_I as i32,
    Int16I   = soxr_datatype_t::SOXR_INT16_I as i32,
    Float32S = soxr_datatype_t::SOXR_FLOAT32_S as i32,
    Float64S = soxr_datatype_t::SOXR_FLOAT64_S as i32,
    Int32S   = soxr_datatype_t::SOXR_INT32_S as i32,
    Int16S   = soxr_datatype_t::SOXR_INT16_S as i32,
    Dynamic,
}

impl DataType {
    pub fn is_packed(&self) -> bool {
        matches!(self, DataType::Float32I | DataType::Float64I | DataType::Int32I | DataType::Int16I)
    }

    pub fn is_planar(&self) -> bool {
        matches!(self, DataType::Float32S | DataType::Float64S | DataType::Int32S | DataType::Int16S)
    }
}

impl TryFrom<DataType> for soxr_datatype_t {
    type Error = Error;

    fn try_from(dt: DataType) -> Result<Self> {
        match dt {
            DataType::Float32I => Ok(soxr_datatype_t::SOXR_FLOAT32_I),
            DataType::Float64I => Ok(soxr_datatype_t::SOXR_FLOAT64_I),
            DataType::Int32I => Ok(soxr_datatype_t::SOXR_INT32_I),
            DataType::Int16I => Ok(soxr_datatype_t::SOXR_INT16_I),
            DataType::Float32S => Ok(soxr_datatype_t::SOXR_FLOAT32_S),
            DataType::Float64S => Ok(soxr_datatype_t::SOXR_FLOAT64_S),
            DataType::Int32S => Ok(soxr_datatype_t::SOXR_INT32_S),
            DataType::Int16S => Ok(soxr_datatype_t::SOXR_INT16_S),
            DataType::Dynamic => Err(Error::with_str(UNSUPPORTED_DATA_TYPE)),
        }
    }
}

impl From<soxr_datatype_t> for DataType {
    fn from(dt: soxr_datatype_t) -> Self {
        match dt {
            soxr_datatype_t::SOXR_FLOAT32_I => DataType::Float32I,
            soxr_datatype_t::SOXR_FLOAT64_I => DataType::Float64I,
            soxr_datatype_t::SOXR_INT32_I => DataType::Int32I,
            soxr_datatype_t::SOXR_INT16_I => DataType::Int16I,
            soxr_datatype_t::SOXR_FLOAT32_S => DataType::Float32S,
            soxr_datatype_t::SOXR_FLOAT64_S => DataType::Float64S,
            soxr_datatype_t::SOXR_INT32_S => DataType::Int32S,
            soxr_datatype_t::SOXR_INT16_S => DataType::Int16S,
        }
    }
}
