use common::{color::RgbaF32, effect::StripInfo, net::StripMode};

use crate::NUM_STRIPS;

pub const MAX_STRIP_LEN: usize = 300;
pub const MAX_STRIP_BUF_LEN: usize = MAX_STRIP_LEN * 24 + 1;

pub struct StripState<const N: usize> {
    pub colors: [RgbaF32; N],
    pub info: StripInfo,
    pub mode: StripMode,
}

#[allow(unused)]
impl<const N: usize> StripState<N> {
    pub const fn new(info: StripInfo) -> Self {
        Self {
            colors: [RgbaF32::zero(); N],
            info,
            mode: StripMode::Hybrid,
        }
    }

    pub const fn empty() -> Self {
        Self {
            colors: [RgbaF32::zero(); N],
            info: StripInfo::empty(),
            mode: StripMode::Off,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.info.leds == 0
    }
}

pub struct State<const BUF_LEN: usize> {
    pub strips: [StripState<BUF_LEN>; NUM_STRIPS],
}

impl<const BUF_LEN: usize> State<BUF_LEN> {
    pub const fn new(strips: [StripState<BUF_LEN>; NUM_STRIPS]) -> Self {
        Self { strips }
    }
}
