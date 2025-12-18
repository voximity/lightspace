use common::{
    color::RgbaF32,
    net::{StripMode, UdpMessage},
};
use embassy_net::{
    Runner, Stack,
    tcp::TcpSocket,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_time::{Duration, Timer};
use esp_println::println;
use esp_radio::wifi::{
    ModeConfig, WifiController, WifiDevice, WifiEvent, WifiStationState, scan::ScanConfig,
    sta::StationConfig, station_state,
};

use crate::{NUM_STRIPS, STATE};

const SSID: &'static str = env!("FW_SSID");
const PASSWORD: &'static str = env!("FW_PASSWORD");

#[embassy_executor::task]
pub async fn connection(mut controller: WifiController<'static>, _stack: Stack<'static>) {
    println!("device capabilities: {:?}", controller.capabilities());

    loop {
        match station_state() {
            WifiStationState::Connected => {
                controller
                    .wait_for_event(WifiEvent::StationDisconnected)
                    .await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => (),
        }

        if !controller.is_started().unwrap_or(false) {
            let sta_config = ModeConfig::Station(
                StationConfig::default()
                    .with_ssid(SSID.into())
                    .with_password(PASSWORD.into()),
            );

            controller.set_config(&sta_config).unwrap();
            println!("starting wifi...");
            controller.start_async().await.unwrap();
            println!("wifi started!");

            println!("scanning...");
            let scan_config = ScanConfig::default().with_max(10);
            let result = controller
                .scan_with_config_async(scan_config)
                .await
                .unwrap();

            for ap in result {
                println!("{:?}", ap);
            }
        }

        println!("about to connect...");

        match controller.connect_async().await {
            Ok(_) => {
                println!("wifi connected!");
            }
            Err(e) => {
                println!("failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await;
            }
        }
    }
}

#[embassy_executor::task]
pub async fn task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
pub async fn show_ipv4(stack: Stack<'static>) {
    loop {
        if let Some(cfg) = stack.config_v4() {
            esp_println::println!("ipv4 from dhcp: {}", cfg.address);
            break;
        }
        Timer::after_millis(500).await;
    }
}

// TODO: unify somewhere else
// fn post_process(color: Rgb8) -> Rgb8 {
//     color.gamma_correct().brightness(0.4)
// }

#[embassy_executor::task]
pub async fn udp_socket(stack: Stack<'static>) {
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buf = [0u8; 8092];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buf = [0u8; 8092];
    let mut buf = [0u8; 8092];

    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buf, &mut tx_meta, &mut tx_buf);
    socket.bind(1337).unwrap();

    'recv: loop {
        let (mut n, _) = socket.recv_from(&mut buf).await.unwrap();
        let mut cursor = buf.as_slice();

        // TODO: may want to eventually confirm that this packet is from a trusted endpoint

        let mut state = STATE.lock().await;
        while n >= 2 {
            let mut msg_size = 0usize;
            let (msg_type, target, rest) = (cursor[0], cursor[1], &cursor[2..]);
            msg_size += 2;

            if !(0..NUM_STRIPS).contains(&(target as usize)) {
                continue 'recv;
            }

            if !matches!(
                state.strips[target as usize].mode,
                StripMode::Dynamic | StripMode::Hybrid
            ) {
                continue 'recv;
            }

            let strip = &mut state.strips[target as usize];

            match UdpMessage::try_from(msg_type) {
                Ok(UdpMessage::SetBufferToMany) => {
                    let leds = strip.info.leds;
                    msg_size += leds * 3;
                    if n < msg_size {
                        continue 'recv;
                    }

                    for (src, dst) in rest[0..(leds * 3)].chunks(3).zip(strip.colors.iter_mut()) {
                        *dst = RgbaF32::new_premultiplied(
                            src[0] as f32 / 255.0,
                            src[1] as f32 / 255.0,
                            src[2] as f32 / 255.0,
                            1.0,
                        );
                    }
                }

                Ok(UdpMessage::SetBufferToSingle) => {
                    msg_size += 3;
                    if n < msg_size {
                        continue 'recv;
                    }

                    let (r, g, b) = (cursor[0], cursor[1], cursor[2]);
                    let src = RgbaF32::new_premultiplied(
                        r as f32 / 255.0,
                        g as f32 / 255.0,
                        b as f32 / 255.0,
                        1.0,
                    );

                    for dst in strip.colors.iter_mut() {
                        *dst = src;
                    }
                }

                Ok(UdpMessage::SetBufferToManyAlpha) => {
                    let leds = strip.info.leds;
                    msg_size += leds * 4;
                    if n < msg_size {
                        continue 'recv;
                    }

                    for (src, dst) in rest[0..(leds * 4)].chunks(4).zip(strip.colors.iter_mut()) {
                        *dst = RgbaF32::new_premultiplied(
                            src[0] as f32 / 255.0,
                            src[1] as f32 / 255.0,
                            src[2] as f32 / 255.0,
                            src[3] as f32 / 255.0,
                        );
                    }
                }

                Err(_) => continue 'recv,

                _ => todo!(),
            }

            // shift cursor forward by consumed msg size
            n -= msg_size;
            cursor = &cursor[msg_size..];
        }
    }
}

#[embassy_executor::task]
pub async fn tcp_socket(stack: Stack<'static>) {
    let mut rx = [0u8; 4096];
    let mut tx = [0u8; 4096];
    let mut socket = TcpSocket::new(stack, &mut rx, &mut tx);

    let /* mut */ _buf = [0u8; 4096];
    loop {
        use embassy_net::tcp::AcceptError;
        match socket.accept(1338).await {
            Ok(_) => {
                // TODO: decode into `ServerMessage`
            }
            Err(AcceptError::ConnectionReset) => {
                println!("warn: reset on TCP connection");
                Timer::after_secs(5).await;
                continue;
            }
            Err(_) => panic!("error: TCP connection fail"),
        }
    }
}
