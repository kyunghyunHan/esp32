#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::{alloc::Layout, mem::MaybeUninit};

use esp_alloc as _;
use esp_hal::{
    clock::CpuClock,
    i2c::master::{Config, I2c},
    main,
    rng::Rng,
    timer::timg::TimerGroup,
    time::Rate,
};
use esp_wifi::wifi::{AuthMethod, ClientConfiguration, Configuration, InternalWifiError, WifiError, WifiState};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[alloc_error_handler]
fn alloc_error(_: Layout) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

const WIFI_SSID: &str = "kyunghyun’s iPhone";
const WIFI_PASSWORD: &str = "25320000";

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

#[main]
fn main() -> ! {
    init_heap();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // I2C 초기화
    let i2c = I2c::new(
        peripherals.I2C0,
        Config::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO21)
    .with_scl(peripherals.GPIO22);

    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    display.init().unwrap();
    display.clear(BinaryColor::Off).unwrap();
    display.flush().unwrap();

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let mut show = |text: &str| {
        display.clear(BinaryColor::Off).unwrap();
        Text::new(text, Point::new(0, 16), style)
            .draw(&mut display)
            .unwrap();
        display.flush().unwrap();
    };

    show("WiFi: init 1");

    // let timg0 = TimerGroup::new(peripherals.TIMG0);
    // show("WiFi: init 2");

    // let rng = Rng::new(peripherals.RNG);
    // show("WiFi: init 3");

    // let esp_wifi_ctrl = match esp_wifi::init(timg0.timer0, rng) {
    //     Ok(v) => v,
    //     Err(_) => {
    //         show("WiFi: init FAIL");
    //         loop {}
    //     }
    // };
    let timg1 = TimerGroup::new(peripherals.TIMG1);
let rng = Rng::new(peripherals.RNG);
let esp_wifi_ctrl = esp_wifi::init(timg1.timer0, rng).unwrap();
    show("WiFi: init 4");

    let (mut controller, _interfaces) = match esp_wifi::wifi::new(&esp_wifi_ctrl, peripherals.WIFI) {
        Ok(v) => v,
        Err(err) => {
            match err {
                WifiError::NotInitialized => show("WiFi: new not init"),
                WifiError::Unsupported => show("WiFi: new UNSUP"),
                WifiError::InvalidArguments => show("WiFi: new args"),
                WifiError::UnknownWifiMode => show("WiFi: new mode"),
                WifiError::InternalError(InternalWifiError::EspErrNoMem) => {
                    show("WiFi: new no mem")
                }
                WifiError::InternalError(InternalWifiError::EspErrWifiInitState) => {
                    show("WiFi: new init st")
                }
                WifiError::InternalError(InternalWifiError::EspErrWifiNvs) => {
                    show("WiFi: new nvs")
                }
                _ => show("WiFi: new err"),
            }
            loop {}
        }
    };
    show("WiFi: init 5");

    let client_config = Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.into(),
        password: WIFI_PASSWORD.into(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    });

    controller.set_configuration(&client_config).unwrap();
    show("WiFi: start");
    controller.start().unwrap();
    show("WiFi: connect");
    controller.connect().unwrap();

    let mut last_state = WifiState::StaDisconnected;

    loop {
        let state = esp_wifi::wifi::sta_state();
        if state != last_state {
            if state == WifiState::StaConnected {
                show("WiFi: connected");
            } else if state == WifiState::StaDisconnected {
                show("WiFi: retrying");
            } else {
                show("WiFi: working");
            }
        }

        if state == WifiState::StaDisconnected {
            let _ = controller.connect();
        }

        last_state = state;
    }
}
