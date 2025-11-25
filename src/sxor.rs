use std::{
    ffi::{c_uint, c_void, CStr},
    marker::PhantomData,
    ptr,
};

use smallvec::SmallVec;

use crate::{
    data_type::DataType,
    error::{Error, Result, INVALID_CHANNELS},
    soxr_sys,
    spec::{IOSpec, QualitySpec, RuntimeSpec},
};

pub trait Sample {
    type ValueType: Copy + 'static;
    const DATA_TYPE: DataType;
}

pub struct Packed<T>(PhantomData<T>);
pub struct Planar<T>(PhantomData<T>);
pub struct DynamicSample;

impl Sample for Packed<f32> {
    type ValueType = f32;
    const DATA_TYPE: DataType = DataType::Float32I;
}

impl Sample for Packed<f64> {
    type ValueType = f64;
    const DATA_TYPE: DataType = DataType::Float64I;
}

impl Sample for Packed<i32> {
    type ValueType = i32;
    const DATA_TYPE: DataType = DataType::Int32I;
}

impl Sample for Packed<i16> {
    type ValueType = i16;
    const DATA_TYPE: DataType = DataType::Int16I;
}

impl Sample for Planar<f32> {
    type ValueType = f32;
    const DATA_TYPE: DataType = DataType::Float32S;
}

impl Sample for Planar<f64> {
    type ValueType = f64;
    const DATA_TYPE: DataType = DataType::Float64S;
}

impl Sample for Planar<i32> {
    type ValueType = i32;
    const DATA_TYPE: DataType = DataType::Int32S;
}

impl Sample for Planar<i16> {
    type ValueType = i16;
    const DATA_TYPE: DataType = DataType::Int16S;
}

impl Sample for DynamicSample {
    type ValueType = u8;
    const DATA_TYPE: DataType = DataType::Dynamic;
}

pub enum SampleBuffer<'a, T: Sample> {
    Packed(&'a [T::ValueType]),
    Planar(&'a [&'a [T::ValueType]]),
}

pub enum SampleBufferMut<'a, T: Sample> {
    Packed(&'a mut [T::ValueType]),
    Planar(&'a mut [&'a mut [T::ValueType]]),
}

const DEFAULT_MAX_CHANNELS: usize = 16;

pub struct Soxr<I: Sample = DynamicSample, O: Sample = DynamicSample> {
    soxr: soxr_sys::soxr_t,
    channels: u8,
    input_data_type: Option<DataType>,
    output_data_type: Option<DataType>,
    _phantom: PhantomData<(I, O)>,
}

impl<I: Sample, O: Sample> Soxr<I, O> {
    pub fn version() -> String {
        unsafe { CStr::from_ptr(soxr_sys::soxr_version()).to_string_lossy().into_owned() }
    }

    pub fn new(
        input_rate: f64,
        output_rate: f64,
        num_channels: u8,
        quality_spec: Option<&QualitySpec>,
        runtime_spec: Option<&RuntimeSpec>,
    ) -> Result<Self> {
        let mut err: soxr_sys::soxr_error_t = ptr::null_mut();
        let io_spec = IOSpec::new(I::DATA_TYPE, O::DATA_TYPE)?;

        let soxr = unsafe {
            soxr_sys::soxr_create(
                input_rate,
                output_rate,
                num_channels as c_uint,
                &mut err,
                &io_spec.io_spec,
                quality_spec.map_or(ptr::null(), |spec| &spec.quality_spec),
                runtime_spec.map_or(ptr::null(), |spec| &spec.runtime_spec),
            )
        };

        if !err.is_null() {
            return Err(Error::new(err));
        }

        Ok(Self {
            soxr,
            channels: num_channels,
            input_data_type: None,
            output_data_type: None,
            _phantom: PhantomData,
        })
    }

    pub fn new_with_data_type(
        input_data_type: DataType,
        output_data_type: DataType,
        input_rate: f64,
        output_rate: f64,
        num_channels: u8,
        quality_spec: Option<&QualitySpec>,
        runtime_spec: Option<&RuntimeSpec>,
    ) -> Result<Self> {
        let mut err: soxr_sys::soxr_error_t = ptr::null_mut();
        let io_spec = IOSpec::new(input_data_type, output_data_type)?;

        let soxr = unsafe {
            soxr_sys::soxr_create(
                input_rate,
                output_rate,
                num_channels as c_uint,
                &mut err,
                &io_spec.io_spec,
                quality_spec.map_or(ptr::null(), |spec| &spec.quality_spec),
                runtime_spec.map_or(ptr::null(), |spec| &spec.runtime_spec),
            )
        };

        if !err.is_null() {
            return Err(Error::new(err));
        }

        Ok(Self {
            soxr,
            channels: num_channels,
            input_data_type: Some(input_data_type),
            output_data_type: Some(output_data_type),
            _phantom: PhantomData,
        })
    }

    fn validate_channels(&self, channels: usize) -> Result<()> {
        if self.channels as usize != channels || channels == 0 {
            Err(Error::with_str(INVALID_CHANNELS))
        } else {
            Ok(())
        }
    }

    unsafe fn process_internal(&mut self, in_ptr: *const c_void, in_len: usize, out_ptr: *mut c_void, out_len: usize) -> Result<(usize, usize)> {
        let mut idone: usize = 0;
        let mut odone: usize = 0;

        let err = unsafe { soxr_sys::soxr_process(self.soxr, in_ptr, in_len, &mut idone, out_ptr, out_len, &mut odone) };

        if !err.is_null() {
            return Err(Error::new(err));
        }

        Ok((idone, odone))
    }

    pub fn process(&mut self, input: Option<SampleBuffer<I>>, output: SampleBufferMut<O>) -> Result<(usize, usize)> {
        let (in_ptr, in_len, _in_vec) = match input {
            Some(SampleBuffer::Packed(buf)) => (buf.as_ptr() as *const _, buf.len() / self.channels as usize, None),
            Some(SampleBuffer::Planar(bufs)) => {
                self.validate_channels(bufs.len())?;
                let samples = bufs[0].len();
                let buf_vec: SmallVec<[_; DEFAULT_MAX_CHANNELS]> = bufs.iter().map(|buf| buf.as_ref().as_ptr()).collect();
                (buf_vec.as_ptr() as *const _, samples, Some(buf_vec))
            }
            None => (ptr::null(), 0, None),
        };

        let (out_ptr, out_len, _out_vec) = match output {
            SampleBufferMut::Packed(buf) => (buf.as_mut_ptr() as *mut _, buf.len() / self.channels as usize, None),
            SampleBufferMut::Planar(bufs) => {
                self.validate_channels(bufs.len())?;
                let samples = bufs[0].len();
                let mut buf_vec: SmallVec<[_; DEFAULT_MAX_CHANNELS]> = bufs.iter_mut().map(|buf| buf.as_mut().as_mut_ptr()).collect();

                (buf_vec.as_mut_ptr() as *mut _, samples, Some(buf_vec))
            }
        };

        unsafe { self.process_internal(in_ptr, in_len, out_ptr, out_len) }
    }

    pub fn process_dynamic<In: Sample, Out: Sample>(
        &mut self,
        input: Option<SampleBuffer<In>>,
        output: SampleBufferMut<Out>,
    ) -> Result<(usize, usize)> {
        if let Some(input_data_type) = self.input_data_type {
            if input.is_some() && input_data_type != In::DATA_TYPE {
                return Err(Error::with_str("input data type mismatch"));
            }
        }

        if let Some(output_data_type) = self.output_data_type {
            if output_data_type != Out::DATA_TYPE {
                return Err(Error::with_str("output data type mismatch"));
            }
        }

        let (in_ptr, in_len, _in_vec) = match input {
            Some(SampleBuffer::Packed(buf)) => (buf.as_ptr() as *const _, buf.len() / self.channels as usize, None),
            Some(SampleBuffer::Planar(bufs)) => {
                self.validate_channels(bufs.len())?;
                let samples = bufs[0].len();
                let buf_vec: SmallVec<[_; DEFAULT_MAX_CHANNELS]> = bufs.iter().map(|buf| buf.as_ref().as_ptr()).collect();
                (buf_vec.as_ptr() as *const _, samples, Some(buf_vec))
            }
            None => (ptr::null(), 0, None),
        };

        let (out_ptr, out_len, _out_vec) = match output {
            SampleBufferMut::Packed(buf) => (buf.as_mut_ptr() as *mut _, buf.len() / self.channels as usize, None),
            SampleBufferMut::Planar(bufs) => {
                self.validate_channels(bufs.len())?;
                let samples = bufs[0].len();
                let mut buf_vec: SmallVec<[_; DEFAULT_MAX_CHANNELS]> = bufs.iter_mut().map(|buf| buf.as_mut().as_mut_ptr()).collect();

                (buf_vec.as_mut_ptr() as *mut _, samples, Some(buf_vec))
            }
        };

        unsafe { self.process_internal(in_ptr, in_len, out_ptr, out_len) }
    }

    pub fn error(&self) -> Option<String> {
        let err = unsafe { soxr_sys::soxr_error(self.soxr) };

        if !err.is_null() {
            return Some(Error::new(err).to_string());
        }

        None
    }

    pub fn num_clips(&self) -> usize {
        unsafe { *soxr_sys::soxr_num_clips(self.soxr) }
    }

    pub fn set_num_clips(&mut self, num_clips: usize) {
        unsafe {
            *soxr_sys::soxr_num_clips(self.soxr) = num_clips;
        }
    }

    pub fn delay(&self) -> f64 {
        unsafe { soxr_sys::soxr_delay(self.soxr) }
    }

    pub fn engine(&self) -> String {
        unsafe { CStr::from_ptr(soxr_sys::soxr_engine(self.soxr)).to_string_lossy().into_owned() }
    }

    pub fn clear(&mut self) -> Result<()> {
        let err = unsafe { soxr_sys::soxr_clear(self.soxr) };

        if !err.is_null() {
            return Err(Error::new(err));
        }

        Ok(())
    }

    pub fn set_io_ratio(&mut self, io_ratio: f64, slew_len: usize) -> Result<()> {
        let err = unsafe { soxr_sys::soxr_set_io_ratio(self.soxr, io_ratio, slew_len) };

        if !err.is_null() {
            return Err(Error::new(err));
        }

        Ok(())
    }

    pub fn set_num_channels(&mut self, num_channels: u32) -> Result<()> {
        let err = unsafe { soxr_sys::soxr_set_num_channels(self.soxr, num_channels) };

        if !err.is_null() {
            return Err(Error::new(err));
        }

        Ok(())
    }
}

impl<I: Sample, O: Sample> Drop for Soxr<I, O> {
    fn drop(&mut self) {
        unsafe { soxr_sys::soxr_delete(self.soxr) }
    }
}
