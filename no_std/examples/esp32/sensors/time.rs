#![no_std]
#![no_main]

use core::fmt::Write;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Pull},
    i2c::master::{Config, I2c},
    main,
    time::Rate,
};
use heapless::String;
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let delay = Delay::new();

    // BOOT 버튼 (GPIO0, 눌리면 LOW)
    let boot = Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    );

    // I2C
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

    // ===== 시계 상태 =====
    let mut h: u8 = 0;
    let mut m: u8 = 0;
    let mut s: u8 = 0;

    let mut running = true;
    let mut was_pressed = false;

    loop {
        // 버튼 토글 (눌렀을 때만)
        let pressed = boot.is_low();
        if pressed && !was_pressed {
            running = !running;
        }
        was_pressed = pressed;

        if running {
            // 1초 기다렸다가 +1
            delay.delay_millis(1000u32);

            s += 1;
            if s == 60 {
                s = 0;
                m += 1;
                if m == 60 {
                    m = 0;
                    h = (h + 1) % 24;
                }
            }

            // 화면 갱신
            let mut buf: String<16> = String::new();
            let _ = write!(buf, "{:02}:{:02}:{:02}", h, m, s);

            display.clear(BinaryColor::Off).unwrap();
            Text::new(&buf, Point::new(0, 16), style)
                .draw(&mut display)
                .unwrap();
            display.flush().unwrap();
        } else {
            // 멈춘 상태에서는 너무 바쁘게 돌지 않게 조금 쉬기
            delay.delay_millis(50u32);
        }
    }
}
