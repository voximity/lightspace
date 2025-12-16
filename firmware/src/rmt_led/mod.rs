mod ws2812b;

use core::convert::Infallible;

use embassy_time::{Duration, Timer};
use embedded_io::{ErrorType, Write};
use esp_hal::{
    Async, Blocking, DriverMode,
    gpio::interconnect::PeripheralOutput,
    rmt::{Channel, Error, PulseCode, Tx, TxChannelConfig, TxChannelCreator},
};
pub use ws2812b::*;

/// An LED protocol.
pub trait RmtLed {
    /// The signal low pulse code.
    const LO: PulseCode;
    /// The signal high pulse code.
    const HI: PulseCode;
    /// The time required for the protocol to "latch" on the new value.
    const LATCH: Duration;

    /// Write a byte to a buffer of pulse codes, at the beginning.
    /// Returns the number of pulse codes written.
    fn write_byte(buf: &mut [PulseCode], byte: u8) -> usize;
}

/// Implement on an LED protocol when a certain color type can be
/// written to the bitstream.
pub trait WriteColor<T> {
    fn write_color(buf: &mut [PulseCode], color: T) -> usize;
}

/// A buffer of RMT pulse codes that can be cleanly written to in sequence.
#[derive(Debug, Clone)]
pub struct RmtBuf<T: RmtLed, const N: usize> {
    buf: [PulseCode; N],
    pos: usize,
    _phantom: core::marker::PhantomData<T>,
}

impl<T: RmtLed, const N: usize> Default for RmtBuf<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RmtLed, const N: usize> RmtBuf<T, N> {
    /// Instantiate a new buffer.
    pub const fn new() -> Self {
        let mut buf = [T::LO; N];
        buf[N - 1] = PulseCode::end_marker();
        Self {
            buf,
            pos: 0,
            _phantom: core::marker::PhantomData,
        }
    }

    /// View into the current buffer.
    pub fn buf(&self) -> &[PulseCode; N] {
        &self.buf
    }

    fn cur_buf_mut(&mut self) -> &mut [PulseCode] {
        self.buf[self.pos..].as_mut()
    }

    /// Write a color into this buffer, if the LED protocol supports it.
    pub fn write_color<C>(&mut self, color: C) -> usize
    where
        T: WriteColor<C>,
    {
        let s = T::write_color(self.cur_buf_mut(), color);
        self.pos += s;
        s
    }
}

impl<'a, T: RmtLed, const N: usize> ErrorType for RmtBuf<T, N> {
    type Error = Infallible;
}

impl<T: RmtLed, const N: usize> Write for RmtBuf<T, N> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let mut written = 0usize;
        for &byte in buf {
            let s = T::write_byte(self.cur_buf_mut(), byte);
            written += s;
            self.pos += s;
        }
        Ok(written)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.pos = 0;
        Ok(())
    }
}

/// An `RmtStrip` wraps an RMT channel.
pub struct RmtStrip<'ch, Dm, T>
where
    Dm: DriverMode,
    T: RmtLed,
{
    pub ch: Channel<'ch, Dm, Tx>,
    _phantom: core::marker::PhantomData<T>,
}

impl<'ch, Dm, T> RmtStrip<'ch, Dm, T>
where
    Dm: DriverMode,
    T: RmtLed,
{
    /// Create a new strip given a RMT channel and an output pin.
    pub fn new_on_channel(
        channel: impl TxChannelCreator<'ch, Dm>,
        pin: impl PeripheralOutput<'ch>,
    ) -> Result<Self, Error> {
        let ch = channel
            .configure_tx(
                // pin,
                &TxChannelConfig::default()
                    .with_idle_output(true)
                    .with_clk_divider(1),
            )
            .unwrap()
            .with_pin(pin);

        Ok(Self {
            ch,
            _phantom: Default::default(),
        })
    }

    /// Wait the latch time defined by the LED protocol.
    #[allow(unused)]
    pub async fn latch(&self) {
        Timer::after(T::LATCH).await;
    }
}

impl<'ch, T: RmtLed> RmtStrip<'ch, Async, T> {
    /// Transmit the current buffer over RMT asynchronously.
    #[allow(unused)]
    pub fn transmit<const SIZE: usize>(
        &mut self,
        buf: &RmtBuf<T, SIZE>,
    ) -> impl Future<Output = Result<(), Error>> {
        self.ch.transmit(buf.buf())
    }
}

impl<'ch, T: RmtLed> RmtStrip<'ch, Blocking, T> {
    /// Transmit the current buffer over RMT, blocking the current thread. Steals ownership of
    /// the strip. Ensure you replace it!
    #[allow(unused)]
    #[must_use]
    pub fn transmit_blocking<'a, const SIZE: usize>(
        mut self,
        buf: &RmtBuf<T, SIZE>,
    ) -> Result<Self, Error> {
        let tx = self.ch.transmit(buf.buf()).unwrap();
        self.ch = tx.wait().map_err(|t| t.0)?; // TODO: is this safe?
        Ok(self)
    }
}
