#![no_std]
#![no_main]

use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Pull},
    i2c::master::{Config, I2c},
    main,
    time::Rate,
};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // PIR 센서 GPIO4 (Pull Down)
    let pir = Input::new(
        peripherals.GPIO27,
        InputConfig::default().with_pull(Pull::Down),
    );

    // I2C OLED
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

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    let mut was_motion = false;

    // delay 객체
    let delay = Delay::new();

    loop {
        let motion = pir.is_high();

        // 상태 변화 있을 때만 OLED 갱신
        if motion != was_motion {
            display.clear(BinaryColor::Off).unwrap();

            let text = if motion { "YES" } else { "NO" };

            Text::new(text, Point::new(40, 32), style)
                .draw(&mut display)
                .unwrap();

            display.flush().unwrap();

            was_motion = motion;
        }

        // CPU 과부하 방지
        delay.delay_millis(50);
    }
}
