use effect::color::Rgb8;
use embassy_net::{
    Runner, Stack,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_time::{Duration, Timer};
use embedded_io::Write;
use esp_println::println;
use esp_radio::wifi::{
    ClientConfig, ModeConfig, ScanConfig, WifiController, WifiDevice, WifiEvent, WifiStaState,
    sta_state,
};

use crate::STRIP0_BUF;

const SSID: &'static str = env!("FW_SSID");
const PASSWORD: &'static str = env!("FW_PASSWORD");

#[embassy_executor::task]
pub async fn connection(mut controller: WifiController<'static>, stack: Stack<'static>) {
    println!("device capabilities: {:?}", controller.capabilities());

    loop {
        match sta_state() {
            WifiStaState::Connected => {
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => (),
        }

        if !controller.is_started().unwrap_or(false) {
            let sta_config = ModeConfig::Client(
                ClientConfig::default()
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
pub async fn udp_socket(stack: Stack<'static>) {
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buf = [0u8; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buf = [0u8; 4096];
    let mut buf = [0u8; 4096];

    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buf, &mut tx_meta, &mut tx_buf);
    socket.bind(1337).unwrap();

    loop {
        let (n, ep) = socket.recv_from(&mut buf).await.unwrap();
        if n == 300 * 3 {
            let mut rmt_buf = STRIP0_BUF.lock().await;
            rmt_buf.flush();
            for slice in buf[0..n].chunks(3) {
                rmt_buf.write_color(
                    Rgb8 {
                        r: slice[0],
                        g: slice[1],
                        b: slice[2],
                    }
                    .gamma_correct()
                    .brightness(0.2),
                );
            }
        }
        socket.send_to(&buf[..n], ep).await.unwrap();
    }
}
