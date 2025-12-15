use core::ops::Deref;

use common::effect::StripInfo;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use crate::rmt_led::{RmtBuf, RmtLed};

/// A `StripMutex` provides concurrent access to a strip's RMT buffer and its strip info.
pub struct StripMutex<T: RmtLed, const N: usize>(
    Mutex<CriticalSectionRawMutex, StripMutexInner<T, N>>,
);

impl<T: RmtLed, const N: usize> StripMutex<T, N> {
    pub const fn new(info: StripInfo) -> Self {
        Self(Mutex::new(StripMutexInner::new(info)))
    }
}

impl<T: RmtLed, const N: usize> Deref for StripMutex<T, N> {
    type Target = Mutex<CriticalSectionRawMutex, StripMutexInner<T, N>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct StripMutexInner<T: RmtLed, const N: usize> {
    buf: RmtBuf<T, N>,
    info: StripInfo,
}

#[allow(unused)]
impl<T: RmtLed, const N: usize> StripMutexInner<T, N> {
    pub const fn new(info: StripInfo) -> Self {
        Self {
            buf: RmtBuf::new(),
            info,
        }
    }

    pub fn buf(&self) -> &RmtBuf<T, N> {
        &self.buf
    }

    pub fn buf_mut(&mut self) -> &mut RmtBuf<T, N> {
        &mut self.buf
    }

    pub fn info(&self) -> &StripInfo {
        &self.info
    }

    pub fn info_mut(&mut self) -> &mut StripInfo {
        &mut self.info
    }
}
