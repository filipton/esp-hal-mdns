#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, delay::Delay, peripherals::Peripherals, prelude::*, system::SystemControl,
};
use esp_wifi::{
    current_millis,
    wifi::{AccessPointInfo, ClientConfiguration, Configuration, WifiError, WifiStaDevice},
    wifi_interface::WifiStack,
};
use smoltcp::{iface::SocketStorage, socket::udp::PacketMetadata};

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

const MULTICAST_ADDR: smoltcp::wire::IpAddress =
    smoltcp::wire::IpAddress::Ipv4(smoltcp::wire::Ipv4Address([224, 0, 0, 251]));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    esp_println::logger::init_logger_from_env();

    let timer = esp_hal::timer::PeriodicTimer::new(
        esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1, &clocks, None)
            .timer0
            .into(),
    );
    let init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        timer,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let wifi = peripherals.WIFI;
    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let (iface, device, mut controller, sockets) = esp_wifi::wifi::utils::create_network_interface(
        &init,
        wifi,
        WifiStaDevice,
        &mut socket_set_entries,
    )
    .unwrap();
    let wifi_stack = WifiStack::new(iface, device, sockets, current_millis);

    let client_config = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        ..Default::default()
    });
    let res = controller.set_configuration(&client_config);
    log::info!("wifi_set_configuration returned {:?}", res);

    controller.start().unwrap();
    log::info!("is wifi started: {:?}", controller.is_started());
    log::info!("{:?}", controller.get_capabilities());
    log::info!("wifi_connect {:?}", controller.connect());

    // wait to get connected
    log::info!("Wait to get connected");
    loop {
        let res = controller.is_connected();
        match res {
            Ok(connected) => {
                if connected {
                    break;
                }
            }
            Err(err) => {
                log::info!("{:?}", err);
                loop {}
            }
        }
    }
    log::info!("{:?}", controller.is_connected());

    // wait for getting an ip address
    log::info!("Wait to get an ip address");
    loop {
        wifi_stack.work();

        if wifi_stack.is_iface_up() {
            log::info!("got ip {:?}", wifi_stack.get_ip_info());
            break;
        }
    }

    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 1536];
    let mut rx_meta = [PacketMetadata::EMPTY; 512];
    let mut tx_meta = [PacketMetadata::EMPTY; 512];
    let mut sock =
        wifi_stack.get_udp_socket(&mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);

    log::info!("sock.bind(5353) res: {:?}", sock.bind(5353));
    log::info!(
        "multicast_res: {:?}",
        sock.join_multicast_group(MULTICAST_ADDR)
    );

    /*
    log::info!(
        "sock.send: {:?}",
        sock.send(MULTICAST_ADDR, 5353, &[69; 420])
    );
    */

    let mut data_buf = [0; 4096];
    loop {
        wifi_stack.work();
        sock.work();
        let res = sock.receive(&mut data_buf);
        if let Ok(res) = res {
            log::info!("sock.receive res: {res:?}");
        }

        delay.delay(50.millis());
    }
}
