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
use common::{
    color::Rgb8,
    effect::{ColorPattern, ColorWheel, StripInfo},
};
use embassy_executor::Spawner;
use embassy_net::StackResources;
use embassy_time::{Instant, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::rmt::Rmt;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{Blocking, rng::Rng};
use esp_rtos::embassy::Executor;
use static_cell::StaticCell;

use crate::{
    fx::Effects,
    rmt_led::{RmtStrip, Ws2812b},
    strip::{MAX_STRIP_BUF_LEN, StripBuf, StripBufs},
};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

#[cfg(feature = "esp32s3")]
pub const NUM_STRIPS: usize = 4;
#[cfg(feature = "esp32c6")]
pub const NUM_STRIPS: usize = 2;

pub static STRIP_BUFS: StripBufs<Ws2812b, MAX_STRIP_BUF_LEN> = StripBufs::new([
    StripBuf::new(StripInfo {
        leds: 300,
        rev: true,
    }),
    StripBuf::new(StripInfo {
        leds: 300,
        rev: false,
    }),
    StripBuf::empty(),
    StripBuf::empty(),
]);

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    static APP_CORE_STACK: StaticCell<esp_hal::system::Stack<8192>> = StaticCell::new();
    let app_core_stack = APP_CORE_STACK.init(esp_hal::system::Stack::new());

    #[cfg(feature = "esp32c6")]
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);
    #[cfg(feature = "esp32s3")]
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let (wifi_controller, interfaces) = esp_radio::wifi::new(peripherals.WIFI, Default::default())
        .expect("Failed to initialize Wi-Fi controller");

    // init embassy net
    let wifi_interface = interfaces.station;
    let config = embassy_net::Config::dhcpv4(Default::default());
    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    let stack_resources = STACK_RESOURCES.init(StackResources::new());
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
    let strips = [
        RmtStrip::new_on_channel(rmt.channel0, peripherals.GPIO4).unwrap(),
        RmtStrip::new_on_channel(rmt.channel1, peripherals.GPIO5).unwrap(),
        #[cfg(feature = "esp32s3")]
        RmtStrip::new_on_channel(rmt.channel2, peripherals.GPIO6).unwrap(),
        #[cfg(feature = "esp32s3")]
        RmtStrip::new_on_channel(rmt.channel3, peripherals.GPIO7).unwrap(),
    ];

    // on the ESP32-S3, we can pin LED data transmission to the second core
    #[cfg(feature = "esp32s3")]
    esp_rtos::start_second_core(
        peripherals.CPU_CTRL,
        sw_interrupt.software_interrupt1,
        app_core_stack,
        move || {
            static EXECUTOR: StaticCell<Executor> = StaticCell::new();
            let executor = EXECUTOR.init(Executor::new());
            executor.run(|spawner| {
                spawner.spawn(data_tx(strips)).unwrap();
            });
        },
    );

    #[cfg(feature = "esp32c6")]
    spawner.spawn(data_tx(strip0, strip1)).unwrap();

    core::future::pending::<()>().await;
    loop {}
}

#[embassy_executor::task]
async fn data_tx(mut strips: [RmtStrip<'static, Blocking, Ws2812b>; NUM_STRIPS]) {
    let _fx = Effects::new([
        Box::new(ColorWheel {
            deg_per_sec: 180.0,
            ..Default::default()
        }),
        Box::new(ColorPattern {
            colors: [Rgb8::new(255, 0, 0), Rgb8::new(0, 255, 0)],
            speed: 1.0,
        }),
    ]);

    let _strip_info = StripInfo {
        leds: 300,
        rev: false,
    };
    let _effect_buf = [Rgb8::zero(); 300];

    loop {
        let _now = esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_millis();

        // fx.update(&strip_info, &mut effect_buf, now);

        // lock our strip buffers
        let mut bufs = STRIP_BUFS.lock().await;

        // transmit the first strip, keeping track of its finish time
        let mut t = None;
        for i in 0..NUM_STRIPS {
            if bufs[i].info.leds == 0 {
                continue;
            }

            strips[i].transmit_blocking(&mut bufs[i].rmt_buf);

            if t.is_none() {
                t = Some(Instant::now());
            }
        }

        // drop the lock
        drop(bufs);

        match t {
            Some(t) => Timer::at(t + <Ws2812b as crate::rmt_led::RmtLed>::LATCH).await,
            None => Timer::after_secs(1).await,
        }
    }
}
