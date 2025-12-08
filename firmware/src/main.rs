#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

mod net;
mod rmt_led;

use effect::{
    color::Rgb8,
    mode::{Bounce, ColorPattern, ColorWheel, EffectMode, StripInfo},
};
use embassy_executor::Spawner;
use embassy_net::StackResources;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use embedded_io::Write;
use esp_hal::clock::CpuClock;
use esp_hal::rmt::Rmt;
use esp_hal::rng::Rng;
use esp_hal::time::{Instant, Rate};
use esp_hal::timer::timg::TimerGroup;
use esp_radio::Controller;
use num_traits::Euclid;
use static_cell::StaticCell;

use crate::rmt_led::{RmtBuf, RmtBufMutex, Strip, Ws2812b};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

static RADIO_CTRL: StaticCell<Controller<'static>> = StaticCell::new();
static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

const STRIP_BUF_LEN: usize = 24 * 300 + 1;
pub static STRIP0_BUF: RmtBufMutex<Ws2812b, STRIP_BUF_LEN> = Mutex::new(RmtBuf::new());
pub static STRIP1_BUF: RmtBufMutex<Ws2812b, STRIP_BUF_LEN> = Mutex::new(RmtBuf::new());

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

    // spawn wifi stuff
    // spawner
    //     .spawn(net::connection(wifi_controller, stack))
    //     .unwrap();
    // spawner.spawn(net::task(runner)).unwrap();
    // spawner.spawn(net::udp_socket(stack)).unwrap();

    // wait until dhcp gives us an ip
    // loop {
    //     if let Some(cfg) = stack.config_v4() {
    //         println!("ip: {}", cfg.address);
    //         break;
    //     }
    //     Timer::after(Duration::from_millis(500)).await;
    // }

    // rmt init
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).expect("failed to initialize RMT");
    // .into_async();

    let mut strip0 = Strip::<_, Ws2812b>::new_on_channel(rmt.channel0, peripherals.GPIO4)
        .expect("failed to init strip");
    let mut strip1 = Strip::<_, Ws2812b>::new_on_channel(rmt.channel1, peripherals.GPIO5)
        .expect("failed to init strip");

    let _ = spawner;

    // TODO: store effects in one location
    let fx = ColorWheel::default();
    // let fx = Bounce {
    //     color: Rgb8::new(255, 0, 0),
    //     speed: 0.2,
    // };
    let strip_info = StripInfo {
        leds: 300,
        rev: false,
    };
    let mut effect_buf = [Rgb8::zero(); 300];

    loop {
        let now = Instant::now().duration_since_epoch().as_millis();
        fx.update(&strip_info, &mut effect_buf, now);

        let mut buf = STRIP0_BUF.lock().await;
        _ = buf.flush();

        for px in effect_buf {
            buf.write_color(px.brightness(0.4));
        }

        // render shared buffer to both strips
        strip0 = strip0.transmit_blocking(&buf).unwrap();
        let t = embassy_time::Instant::now();
        strip1 = strip1.transmit_blocking(&buf).unwrap();
        Timer::at(t + <Ws2812b as crate::rmt_led::RmtLed>::LATCH).await;
    }
}
