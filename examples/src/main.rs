#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;
use core::str::FromStr;
use embassy_executor::Spawner;
use embassy_net::{tcp::TcpSocket, Config, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{prelude::*, timer::timg::TimerGroup};
use esp_hal_ota::Ota;
use esp_storage::FlashStorage;
use esp_wifi::{
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
    EspWifiController,
};

const WIFI_SSID: &'static str = env!("SSID");
const WIFI_PSK: &'static str = env!("PSK");
const OTA_SERVER_IP: &'static str = env!("OTA_IP");

const RX_BUFFER_SIZE: usize = 16384;
const TX_BUFFER_SIZE: usize = 16384;
static mut TX_BUFF: [u8; TX_BUFFER_SIZE] = [0; TX_BUFFER_SIZE];
static mut RX_BUFF: [u8; RX_BUFFER_SIZE] = [0; RX_BUFFER_SIZE];

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[main]
async fn main(spawner: Spawner) {
    esp_alloc::heap_allocator!(150 * 1024);

    let peripherals = esp_hal::init(esp_hal::Config::default());
    //let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    esp_println::logger::init_logger_from_env();
    //log::set_max_level(log::LevelFilter::Info); // only for esp32s3??

    let rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let timg1 = TimerGroup::new(peripherals.TIMG1);

    let init = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timg1.timer0, rng.clone(), peripherals.RADIO_CLK).unwrap()
    );

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

    let config = Config::dhcpv4(Default::default());
    let seed = 69420;

    let stack = &*mk_static!(
        Stack<WifiDevice<'_, WifiStaDevice>>,
        Stack::new(
            wifi_interface,
            config,
            mk_static!(StackResources<3>, StackResources::<3>::new()),
            seed,
        )
    );

    spawner
        .spawn(connection(controller, stack))
        .expect("connection spawn");
    spawner.spawn(net_task(stack)).expect("net task spawn");

    loop {
        log::info!("Wait for wifi!");
        Timer::after(Duration::from_secs(1)).await;

        if let Some(config) = stack.config_v4() {
            log::info!("Got IP: {}", config.address);
            break;
        }
    }

    let mut socket = unsafe {
        TcpSocket::new(
            stack,
            &mut *core::ptr::addr_of_mut!(RX_BUFF),
            &mut *core::ptr::addr_of_mut!(TX_BUFF),
        )
    };

    let ip = embassy_net::IpEndpoint::from_str(OTA_SERVER_IP).expect("Wrong ip addr");
    socket.connect(ip).await.expect("Cannot connect!");
    let mut ota_buff = [0; 4096 * 2];
    socket
        .read(&mut ota_buff[..4])
        .await
        .expect("Cannot read firmware size!");
    let flash_size = u32::from_le_bytes(ota_buff[..4].try_into().unwrap());

    socket
        .read(&mut ota_buff[..4])
        .await
        .expect("Cannot read target crc!");
    let target_crc = u32::from_le_bytes(ota_buff[..4].try_into().unwrap());

    log::info!("flash_size: {flash_size}");
    log::info!("target_crc: {target_crc}");

    let mut ota = Ota::new(FlashStorage::new()).expect("Cannot create ota");
    ota.ota_begin(flash_size, target_crc).unwrap();

    let mut bytes_read = 0;
    loop {
        let res = socket.read(&mut ota_buff).await;
        if let Ok(n) = res {
            bytes_read += n;
            if n == 0 {
                break;
            }

            let res = ota.ota_write_chunk(&ota_buff[..n]);
            if bytes_read % 4096 * 2 == 0 {
                _ = socket.write(&[0]).await;
            }

            match res {
                Ok(true) => {
                    let res = ota.ota_flush(false);
                    if let Err(e) = res {
                        log::error!("Ota flush error: {e:?}");
                        break;
                    }

                    log::info!("Ota OK! Rebooting in 1s!");
                    Timer::after_millis(1000).await;
                    esp_hal::reset::software_reset();
                    break;
                }
                Err(e) => {
                    log::error!("Ota write error: {e:?}");
                    break;
                }
                _ => {}
            }
        }

        Timer::after_millis(10).await;
        log::info!("Progress: {}%", (ota.get_ota_progress() * 100.0) as u8);
    }

    loop {
        log::info!("bump");
        Timer::after_millis(15000).await;
    }
}

#[embassy_executor::task]
async fn connection(
    mut controller: WifiController<'static>,
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
) {
    log::info!("start connection task");
    log::info!("Device capabilities: {:?}", controller.capabilities());
    loop {
        if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
            // wait until we're no longer connected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: WIFI_SSID.try_into().expect("Wifi ssid parse"),
                password: WIFI_PSK.try_into().expect("Wifi psk parse"),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            log::info!("Starting wifi");
            controller.start_async().await.unwrap();
            log::info!("Wifi started!");
        }
        log::info!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => {
                log::info!("Wifi connected!");

                loop {
                    if stack.is_link_up() {
                        break;
                    }
                    Timer::after(Duration::from_millis(500)).await;
                }
            }
            Err(e) => {
                log::info!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}
