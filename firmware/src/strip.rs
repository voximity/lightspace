use core::ops::Deref;

use common::effect::StripInfo;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use crate::{
    NUM_STRIPS,
    rmt_led::{RmtBuf, RmtLed},
};

pub const MAX_STRIP_LEN: usize = 500;
pub const MAX_STRIP_BUF_LEN: usize = MAX_STRIP_LEN * 24 + 1;

pub struct StripBuf<T: RmtLed, const N: usize> {
    pub rmt_buf: RmtBuf<T, N>,
    pub info: StripInfo,
}

#[allow(unused)]
impl<T: RmtLed, const N: usize> StripBuf<T, N> {
    pub const fn new(info: StripInfo) -> Self {
        Self {
            rmt_buf: RmtBuf::new(info.leds),
            info,
        }
    }

    pub const fn empty() -> Self {
        Self {
            rmt_buf: RmtBuf::empty(),
            info: StripInfo::empty(),
        }
    }
}

pub struct StripBufs<T: RmtLed, const BUF_LEN: usize>(
    pub Mutex<CriticalSectionRawMutex, [StripBuf<T, BUF_LEN>; NUM_STRIPS]>,
);

impl<T: RmtLed, const BUF_LEN: usize> StripBufs<T, BUF_LEN> {
    pub const fn new(bufs: [StripBuf<T, BUF_LEN>; NUM_STRIPS]) -> Self {
        Self(Mutex::new(bufs))
    }
}

impl<T: RmtLed, const BUF_LEN: usize> Deref for StripBufs<T, BUF_LEN> {
    type Target = Mutex<CriticalSectionRawMutex, [StripBuf<T, BUF_LEN>; NUM_STRIPS]>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
