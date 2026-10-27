#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::format;
use core::{alloc::Layout, mem::MaybeUninit};
use esp_alloc as _;

use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    i2c::master::{Config as I2cConfig, I2c},
    main,
    time::Rate,
};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

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

    // ================= OLED =================
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

    // ================= ADC =================
    let mut adc_config = AdcConfig::default();
    let mut pot = adc_config.enable_pin(peripherals.GPIO34, Attenuation::_11dB);

    let mut adc = Adc::new(peripherals.ADC1, adc_config);

    let mut last_percent: u32 = 0;

    // ================= LOOP =================
    loop {
        // 🔥 16회 평균 필터
        let mut sum: u32 = 0;
        for _ in 0..16 {
            let val: u16 = block!(adc.read_oneshot(&mut pot)).unwrap();
            sum += val as u32;
        }
        let raw = (sum / 16) as u16;

        let percent = ((raw as u32 * 100) / 4095).clamp(1, 100);

        // 🔥 작은 변화는 무시 (노이즈 제거)
        if (percent as i32 - last_percent as i32).abs() >= 2 {
            last_percent = percent;

            display.clear(BinaryColor::Off).unwrap();

            let text = format!("{} %", percent);

            Text::new(&text, Point::new(0, 16), style)
                .draw(&mut display)
                .unwrap();

            display.flush().unwrap();
        }

        delay.delay_millis(100u32);
    }
}
