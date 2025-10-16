use std::{
    ffi::CStr,
    fmt::{self, Debug, Display, Formatter},
    str,
};

use crate::soxr_sys;

pub(crate) const INVALID_ERROR_STRING: &str = "invalid error string";
pub(crate) const INVALID_CHANNELS: &str = "invalid number of channels";

pub struct Error {
    err: soxr_sys::soxr_error_t,
    err_str: Option<&'static str>,
}

impl Error {
    pub fn new(err: soxr_sys::soxr_error_t) -> Self {
        Self {
            err,
            err_str: None,
        }
    }

    pub fn with_str(str: &'static str) -> Self {
        Self {
            err: std::ptr::null(),
            err_str: Some(str),
        }
    }

    pub fn as_str(&self) -> &str {
        if !self.err.is_null() {
            return unsafe { str::from_utf8(CStr::from_ptr(self.err).to_bytes()).unwrap_or(INVALID_ERROR_STRING) };
        }

        if let Some(str) = self.err_str {
            return str;
        }

        INVALID_ERROR_STRING
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Error").field(&self.as_str()).finish()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
