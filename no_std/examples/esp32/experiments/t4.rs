#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::{alloc::Layout, fmt::Write, mem::MaybeUninit};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    i2c::master::{Config as I2cConfig, I2c},
    main,
    time::Rate,
};
use heapless::String;
use nb::block;
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

    let delay = Delay::new();

    // ----------------------------
    // OLED (I2C)
    // SDA = GPIO21
    // SCL = GPIO22
    // ----------------------------
    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
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

    // ----------------------------
    // ADC
    // GPIO34 에 배터리 분압 중간점 연결
    // ----------------------------
    let mut adc_config = AdcConfig::default();
    let mut bat_pin = adc_config.enable_pin(peripherals.GPIO34, Attenuation::_11dB);
    let mut adc = Adc::new(peripherals.ADC1, adc_config);

    loop {
        let raw: u16 = block!(adc.read_oneshot(&mut bat_pin)).unwrap();

        // ADC 핀에 들어온 전압 계산
        let adc_voltage = raw_to_adc_voltage(raw);

        // 분압(100k + 100k) 보정 => 실제 배터리 전압은 2배
        let battery_voltage = adc_voltage * 2.0;

        let percent = voltage_to_percent(battery_voltage);

        let mut line1: String<32> = String::new();
        let mut line2: String<32> = String::new();
        let mut line3: String<32> = String::new();

        let _ = write!(line1, "RAW: {}", raw);
        let _ = write!(line2, "BAT: {:.2}V", battery_voltage);
        let _ = write!(line3, "LEVEL: {}%", percent);

        display.clear(BinaryColor::Off).unwrap();

        Text::new("Battery Monitor", Point::new(0, 10), style)
            .draw(&mut display)
            .unwrap();

        Text::new(&line1, Point::new(0, 24), style)
            .draw(&mut display)
            .unwrap();

        Text::new(&line2, Point::new(0, 38), style)
            .draw(&mut display)
            .unwrap();

        Text::new(&line3, Point::new(0, 52), style)
            .draw(&mut display)
            .unwrap();

        display.flush().unwrap();

        delay.delay_millis(1000);
    }
}

// ADC raw (0~4095)를 ADC 핀 전압으로 변환
fn raw_to_adc_voltage(raw: u16) -> f32 {
    // 단순 계산용
    // 실제 보정은 보드/환경 따라 조금 오차 있을 수 있음
    (raw as f32 / 4095.0) * 3.3
}

// 배터리 전압 -> 대략적인 퍼센트
fn voltage_to_percent(v: f32) -> u8 {
    if v >= 4.15 {
        100
    } else if v >= 4.05 {
        90
    } else if v >= 3.95 {
        80
    } else if v >= 3.87 {
        70
    } else if v >= 3.80 {
        60
    } else if v >= 3.74 {
        50
    } else if v >= 3.68 {
        40
    } else if v >= 3.60 {
        30
    } else if v >= 3.50 {
        20
    } else if v >= 3.40 {
        10
    } else {
        0
    }
}
