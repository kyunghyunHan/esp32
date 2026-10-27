#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::string::String;
use core::{alloc::Layout, mem::MaybeUninit};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use esp_alloc as _;
use esp_hal::{
    clock::CpuClock,
    i2c::master::{Config, I2c},
    main,
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_radio::wifi::{
    self, AuthMethod, ClientConfig, InternalWifiError, ModeConfig, WifiError, WifiStaState,
};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

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

esp_bootloader_esp_idf::esp_app_desc!();

const WIFI_SSID: &str = "kyunghyun’s iPhone";
const WIFI_PASSWORD: &str = "25320001";

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

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
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

    // esp-radio는 esp-rtos 스케줄러가 먼저 필요
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);
    show("WiFi: RTOS ok");

    let radio = match esp_radio::init() {
        Ok(v) => v,
        Err(_) => {
            show("Radio init FAIL");
            loop {
                core::hint::spin_loop();
            }
        }
    };
    show("Radio: init ok");

    let (mut controller, _interfaces) =
        match wifi::new(&radio, peripherals.WIFI, wifi::Config::default()) {
            Ok(v) => v,
            Err(err) => {
                match err {
                    WifiError::NotInitialized => show("WiFi: new not init"),
                    WifiError::Unsupported => show("WiFi: new UNSUP"),
                    WifiError::InvalidArguments => show("WiFi: new args"),
                    WifiError::UnknownWifiMode => show("WiFi: new mode"),
                    WifiError::InternalError(InternalWifiError::NoMem) => show("WiFi: new no mem"),
                    _ => show("WiFi: new err"),
                }
                loop {
                    core::hint::spin_loop();
                }
            }
        };
    show("WiFi: new ok");

    let client = ClientConfig::default()
        .with_ssid(String::from(WIFI_SSID))
        .with_password(String::from(WIFI_PASSWORD))
        .with_auth_method(AuthMethod::Wpa2Personal);

    controller.set_config(&ModeConfig::Client(client)).unwrap();

    show("WiFi: start");
    controller.start().unwrap();

    show("WiFi: connect");
    controller.connect().unwrap();

    let mut last_state = WifiStaState::Disconnected;

    loop {
        let state = wifi::sta_state();

        if state != last_state {
            match state {
                WifiStaState::Connected => show("WiFi: connected"),
                WifiStaState::Disconnected => show("WiFi: retrying"),
                _ => show("WiFi: working"),
            }
        }

        if state == WifiStaState::Disconnected {
            let _ = controller.connect();
        }

        last_state = state;
    }
}
