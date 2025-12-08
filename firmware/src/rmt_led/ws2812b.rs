use effect::color::Rgb8;
use embassy_time::Duration;
use esp_hal::{gpio::Level, rmt::PulseCode};

use crate::rmt_led::{RmtLed, WriteColor};

/// WS2812B LEDs.
pub enum Ws2812b {}

impl RmtLed for Ws2812b {
    const LO: PulseCode = PulseCode::new(Level::High, 28, Level::Low, 64);
    const HI: PulseCode = PulseCode::new(Level::High, 56, Level::Low, 48);
    const LATCH: Duration = Duration::from_micros(300);

    fn write_byte(buf: &mut [PulseCode], mut byte: u8) -> usize {
        for i in 0..8 {
            buf[i] = match byte & 0b1000_0000 {
                0 => Self::LO,
                _ => Self::HI,
            };
            byte <<= 1;
        }

        8
    }
}

impl WriteColor<Rgb8> for Ws2812b {
    fn write_color(buf: &mut [PulseCode], Rgb8 { r, g, b }: Rgb8) -> usize {
        Self::write_byte(buf, g);
        Self::write_byte(buf[8..].as_mut(), r);
        Self::write_byte(buf[16..].as_mut(), b);
        24
    }
}
