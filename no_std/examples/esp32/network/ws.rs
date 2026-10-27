#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
use esp_println::println;
extern crate alloc;

use core::{alloc::Layout, mem::MaybeUninit};

use esp_alloc as _;
use esp_bootloader_esp_idf::esp_app_desc;
use esp_hal::{clock::CpuClock, main, timer::timg::TimerGroup};
use esp_radio::wifi::{self, AuthMethod, ClientConfig, ModeConfig, WifiStaState};
use smoltcp::{
    iface::{Config as IfaceConfig, Interface, SocketSet, SocketStorage},
    socket::{dhcpv4, tcp},
    time::Instant as SmolInstant,
    wire::{EthernetAddress, HardwareAddress, IpCidr},
};

esp_app_desc!();

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[alloc_error_handler]
fn alloc_error(_: Layout) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

const WIFI_SSID: &str = "Test2";
const WIFI_PASSWORD: &str = "253200000";

fn init_heap() {
    const HEAP_SIZE: usize = 96 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        let heap_ptr = core::ptr::addr_of_mut!(HEAP) as *mut u8;
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
            heap_ptr,
            HEAP_SIZE,
            esp_alloc::MemoryCapability::Internal.into(),
        ));
    }
}

fn timestamp() -> SmolInstant {
    SmolInstant::from_micros(
        esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_micros() as i64,
    )
}

fn create_interface(device: &mut esp_radio::wifi::WifiDevice) -> Interface {
    let config = IfaceConfig::new(HardwareAddress::Ethernet(EthernetAddress::from_bytes(
        &device.mac_address(),
    )));
    Interface::new(config, device, timestamp())
}

#[main]
fn main() -> ! {
    init_heap();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    let radio = esp_radio::init().unwrap();

    let (mut controller, interfaces) =
        wifi::new(&radio, peripherals.WIFI, wifi::Config::default()).unwrap();
    let mut device = interfaces.sta;

    let client = ClientConfig::default()
        .with_ssid(WIFI_SSID.into())
        .with_password(WIFI_PASSWORD.into())
        .with_auth_method(AuthMethod::Wpa2Personal);

    controller.set_config(&ModeConfig::Client(client)).unwrap();
    controller.start().unwrap();
    controller.connect().unwrap();

    while wifi::sta_state() != WifiStaState::Connected {}

    let mut iface = create_interface(&mut device);

    static mut SOCKETS: [SocketStorage<'static>; 2] = [SocketStorage::EMPTY; 2];
    let mut sockets = SocketSet::new(unsafe { &mut SOCKETS[..] });

    let dhcp_handle = sockets.add(dhcpv4::Socket::new());

    static mut TCP_RX: [u8; 2048] = [0; 2048];
    static mut TCP_TX: [u8; 2048] = [0; 2048];
    let tcp_socket = tcp::Socket::new(
        tcp::SocketBuffer::new(unsafe { &mut TCP_RX[..] }),
        tcp::SocketBuffer::new(unsafe { &mut TCP_TX[..] }),
    );
    let tcp_handle = sockets.add(tcp_socket);

    let mut has_ip = false;

    loop {
        let now = timestamp();
        let _ = iface.poll(now, &mut device, &mut sockets);

        if let Some(event) = sockets.get_mut::<dhcpv4::Socket>(dhcp_handle).poll() {
            match event {
                dhcpv4::Event::Configured(config) => {
                    let cidr: IpCidr = config.address.into();

                    iface.update_ip_addrs(|addrs| {
                        if addrs.iter().next().is_none() {
                            let _ = addrs.push(cidr);
                        } else {
                            addrs[0] = cidr;
                        }
                    });

                    if let Some(router) = config.router {
                        let _ = iface.routes_mut().add_default_ipv4_route(router);
                    }

                    println!("Got IP: {:?}", config.address);
                    has_ip = true;
                }
                dhcpv4::Event::Deconfigured => {
                    has_ip = false;
                    iface.update_ip_addrs(|addrs| addrs.clear());
                    let _ = iface.routes_mut().remove_default_ipv4_route();
                }
            }
        }

        if !has_ip {
            continue;
        }

        {
            let socket = sockets.get_mut::<tcp::Socket>(tcp_handle);
            if !socket.is_open() {
                let _ = socket.listen(8000);
            }

            if socket.can_recv() {
                let _ = socket.recv(|buf| (buf.len(), ()));
                let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n<!doctype html><html><body><h1>Hello from Rust ESP32</h1></body></html>";
                let _ = socket.send_slice(response);
                socket.close();
            }
        }
    }
}
