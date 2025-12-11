#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

mod fx;
mod net;
mod rmt_led;
mod strip;

use alloc::boxed::Box;
use effect::{
    color::Rgb8,
    mode::{ColorPattern, ColorWheel, StripInfo},
};
use embassy_executor::Spawner;
use embassy_net::StackResources;
use embassy_time::Timer;
use embedded_io::Write;
use esp_hal::clock::CpuClock;
use esp_hal::rmt::Rmt;
use esp_hal::rng::Rng;
use esp_hal::time::{Instant, Rate};
use esp_hal::timer::timg::TimerGroup;
use esp_radio::Controller;
use static_cell::StaticCell;

use crate::{
    fx::Effects,
    rmt_led::{RmtStrip, Ws2812b},
    strip::StripMutex,
};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

static RADIO_CTRL: StaticCell<Controller<'static>> = StaticCell::new();
static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

const STRIP_BUF_LEN: usize = 24 * 300 + 1;
// pub static STRIP0_BUF: RmtBufMutex<Ws2812b, STRIP_BUF_LEN> = Mutex::new(RmtBuf::new());
// pub static STRIP1_BUF: RmtBufMutex<Ws2812b, STRIP_BUF_LEN> = Mutex::new(RmtBuf::new());

pub static STRIP0: StripMutex<Ws2812b, STRIP_BUF_LEN> = StripMutex::new(StripInfo {
    leds: 300,
    rev: true,
});
pub static STRIP1: StripMutex<Ws2812b, STRIP_BUF_LEN> = StripMutex::new(StripInfo {
    leds: 300,
    rev: false,
});

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // init radio stuff
    let radio_init = &*RADIO_CTRL.init_with(|| esp_radio::init().unwrap());
    let (wifi_controller, interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    // init embassy net
    let wifi_interface = interfaces.sta;
    let config = embassy_net::Config::dhcpv4(Default::default());
    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    let stack_resources = STACK_RESOURCES.init(StackResources::<3>::new());
    let (stack, runner) = embassy_net::new(wifi_interface, config, stack_resources, seed);

    #[cfg(not(feature = "offline"))]
    {
        spawner
            .spawn(net::connection(wifi_controller, stack))
            .unwrap();
        spawner.spawn(net::task(runner)).unwrap();
        spawner.spawn(net::udp_socket(stack)).unwrap();
        spawner.spawn(net::show_ipv4(stack)).unwrap();
    }

    // rmt init
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).expect("failed to initialize RMT");

    let mut strip0 = RmtStrip::<_, Ws2812b>::new_on_channel(rmt.channel0, peripherals.GPIO4)
        .expect("failed to init strip");
    let mut strip1 = RmtStrip::<_, Ws2812b>::new_on_channel(rmt.channel1, peripherals.GPIO5)
        .expect("failed to init strip");

    let _ = spawner;

    let fx = Effects::new([
        Box::new(ColorWheel::default()),
        Box::new(ColorPattern {
            colors: [Rgb8::new(255, 0, 0), Rgb8::new(0, 255, 0)],
            speed: 1.0,
        }),
    ]);

    let strip_info = StripInfo {
        leds: 300,
        rev: false,
    };
    let mut effect_buf = [Rgb8::zero(); 300];

    loop {
        let now = Instant::now().duration_since_epoch().as_millis();
        fx.update(&strip_info, &mut effect_buf, now);

        let mut s0 = STRIP0.lock().await;
        let mut s1 = STRIP1.lock().await;

        let s0b = s0.buf_mut();
        let s1b = s1.buf_mut();

        _ = s0b.flush();
        _ = s1b.flush();
        for px in effect_buf {
            s0b.write_color(px.brightness(0.2));
            s1b.write_color(px.brightness(0.2));
        }

        let t = embassy_time::Instant::now();
        strip0 = strip0.transmit_blocking(&s0b).unwrap();
        strip1 = strip1.transmit_blocking(&s1b).unwrap();
        drop(s0);
        drop(s1);

        Timer::at(t + <Ws2812b as crate::rmt_led::RmtLed>::LATCH * 2).await;
    }
}
