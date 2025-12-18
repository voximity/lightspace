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
    net::StripMode,
};
use embassy_executor::Spawner;
use embassy_net::StackResources;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
// use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embedded_io::Write;
use esp_backtrace as _;
use esp_hal::rmt::Rmt;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{Blocking, rng::Rng};
use esp_hal::{clock::CpuClock, delay::Delay};
use esp_rtos::embassy::Executor;
use static_cell::StaticCell;

use crate::{
    fx::Effects,
    rmt_led::{RmtBuf, RmtStrip, Ws2812b},
    strip::{MAX_STRIP_BUF_LEN, MAX_STRIP_LEN, State, StripState},
};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

#[cfg(feature = "esp32s3")]
pub const NUM_STRIPS: usize = 2; // TODO: can we shift memory around a bit to hit 4?
#[cfg(feature = "esp32c6")]
pub const NUM_STRIPS: usize = 2;

pub static STATE: Mutex<CriticalSectionRawMutex, State<MAX_STRIP_LEN>> = Mutex::new(State::new([
    StripState::new(StripInfo {
        leds: 300,
        rev: true,
    }),
    StripState::new(StripInfo {
        leds: 300,
        rev: false,
    }),
]));

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    #[cfg(feature = "esp32c6")]
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);
    #[cfg(feature = "esp32s3")]
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

    static APP_CORE_STACK: StaticCell<esp_hal::system::Stack<8192>> = StaticCell::new();
    let app_core_stack = APP_CORE_STACK.init(esp_hal::system::Stack::new());

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
        RmtStrip::<_, Ws2812b>::new_on_channel(rmt.channel0, peripherals.GPIO4).unwrap(),
        RmtStrip::new_on_channel(rmt.channel1, peripherals.GPIO5).unwrap(),
    ];

    static RMT_BUFS: StaticCell<[RmtBuf<Ws2812b, MAX_STRIP_BUF_LEN>; NUM_STRIPS]> =
        StaticCell::new();
    static EFFECT_BUFS: StaticCell<[[Rgb8; MAX_STRIP_LEN]; NUM_STRIPS]> = StaticCell::new();

    let rmt_bufs = {
        let state = STATE.lock().await;
        RMT_BUFS.init_with(|| {
            core::array::from_fn::<_, NUM_STRIPS, _>(|i| {
                RmtBuf::<Ws2812b, MAX_STRIP_BUF_LEN>::new(state.strips[i].info.leds)
            })
        })
    };

    let effect_bufs = EFFECT_BUFS.init_with(|| [[Rgb8::zero(); MAX_STRIP_LEN]; NUM_STRIPS]);

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
                spawner
                    .spawn(data_tx(strips, rmt_bufs, effect_bufs))
                    .unwrap()
            });
        },
    );

    #[cfg(feature = "esp32c6")]
    spawner
        .spawn(data_tx(strips, rmt_bufs, effect_bufs))
        .unwrap();

    core::future::pending::<()>().await;
    loop {}
}

#[embassy_executor::task]
async fn data_tx(
    mut strips: [RmtStrip<'static, Blocking, Ws2812b>; NUM_STRIPS],
    rmt_bufs: &'static mut [RmtBuf<Ws2812b, MAX_STRIP_BUF_LEN>; NUM_STRIPS],
    effect_bufs: &'static mut [[Rgb8; MAX_STRIP_LEN]; NUM_STRIPS],
) {
    use esp_hal::time::Instant;

    let fx = Effects::new([
        Box::new(ColorWheel::default()),
        Box::new(ColorPattern {
            colors: [Rgb8::new(255, 0, 0), Rgb8::new(0, 255, 0)],
            speed: 1.0,
        }),
    ]);

    let delay = Delay::new();
    loop {
        let now = esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_millis();

        let mut state = STATE.lock().await;

        // blend in rgba from state and add to rmt_bufs
        for i in 0..NUM_STRIPS {
            let strip_state = &mut state.strips[i];
            if strip_state.is_empty() {
                continue;
            }

            match strip_state.mode {
                StripMode::Effects | StripMode::Hybrid => {
                    fx.update(&strip_state.info, &mut effect_bufs[i], now)
                }
                _ => effect_bufs[i].fill(Rgb8::zero()),
            }

            // TODO: effects should probably output to a RgbaF32 buffer, making this less important
            match strip_state.mode {
                StripMode::Dynamic | StripMode::Hybrid => {
                    for (rgba, rgb) in state.strips[i].colors.iter().zip(effect_bufs[i].iter_mut())
                    {
                        *rgb = rgba.blend_over((*rgb).into()).into();
                    }
                }
                _ => (),
            }

            // fill rmt bufs
            let rmt_buf = &mut rmt_bufs[i];
            _ = rmt_buf.flush();
            for &rgb in &effect_bufs[i] {
                rmt_buf.write_color(rgb.gamma_correct().brightness(0.5));
            }
        }

        // transmit the first strip, keeping track of its finish time
        let mut t = None;
        for i in 0..NUM_STRIPS {
            if state.strips[i].info.leds == 0 {
                continue;
            }

            strips[i].transmit_blocking(&mut rmt_bufs[i]);

            if t.is_none() {
                t = Some(Instant::now());
            }
        }

        drop(state);

        match t {
            Some(t) => delay.delay_micros((Instant::now() - t).as_micros() as u32),
            None => delay.delay_millis(1000),
        }
    }
}
