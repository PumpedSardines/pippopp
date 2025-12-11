#[repr(C)]
struct SbiRet {
    pub error: u32,
    pub value: u32,
}

impl SbiRet {
    /// SAFETY: The caller must ensure that the error code is valid.
    pub unsafe fn into_result(self) -> Result<u32, SbiError> {
        if self.error == 0 {
            Ok(self.value)
        } else {
            Err(SbiError::new_unchecked(self.error))
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum SbiError {
    Failed = -1,
    NotSupported = -2,
    InvalidParam = -3,
    Denied = -4,
    InvalidAddress = -5,
    AlreadyAvailable = -6,
    AlreadyStarted = -7,
    AlreadyStopped = -8
}

impl SbiError {
    /// SAFETY: The caller must ensure that the error code is valid.
    pub unsafe fn new_unchecked(value: u32) -> Self {
        core::mem::transmute(value as i32 as i8)
    }
}

pub type SbiResult = Result<u32, SbiError>;

pub mod harth;
pub mod base;
pub mod debug_console;
pub mod timer;
