use alloc::boxed::Box;
use effect::{
    color::Rgb8,
    mode::{EffectMode, StripInfo},
};

/// `Effects` stores configured `EffectMode`s and allows shifting between them.
pub struct Effects<const N: usize> {
    list: [Box<dyn EffectMode>; N],
    index: usize,
}

#[allow(unused)]
impl<const N: usize> Effects<N> {
    pub fn new(list: [Box<dyn EffectMode>; N]) -> Self {
        Self { list, index: 0 }
    }

    pub fn set_effect(&mut self, index: usize) {
        self.index = index % N;
    }

    pub fn next_effect(&mut self) {
        self.index = (self.index + 1) % N;
    }

    pub fn prev_effect(&mut self) {
        self.index = self.index.checked_sub(1).unwrap_or(N - 1);
    }

    pub fn current_effect(&self) -> &dyn EffectMode {
        &*self.list[self.index]
    }

    pub fn index(&self) -> usize {
        self.index
    }

    /// Update a color buffer with the current effect.
    pub fn update(&self, strip_info: &StripInfo, buf: &mut [Rgb8], now: u64) {
        self.current_effect().update(strip_info, buf, now)
    }
}
