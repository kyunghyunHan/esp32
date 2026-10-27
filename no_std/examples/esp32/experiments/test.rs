#![no_std]
#![no_main]

use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{DriveMode, Input, InputConfig, Pull},
    i2c::master::{Config as I2cConfig, I2c},
    ledc::{
        LSGlobalClkSource, Ledc, LowSpeed,
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
    },
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

    // ✅ BOOT 버튼: GPIO0 (눌리면 LOW)
    let boot = Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    );

    // =========================
    // ✅ OLED (SSD1306 I2C)
    // =========================
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

    // =========================
    // ✅ BUZZER (PWM: LEDC)
    // =========================
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut timer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    timer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty10Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(2_000), // 🔊 2kHz (필요하면 1000~4000 바꿔보기)
        })
        .unwrap();

    let mut ch0 = ledc.channel(channel::Number::Channel0, peripherals.GPIO25);
    ch0.configure(channel::config::Config {
        timer: &timer0,
        duty_pct: 50,
        drive_mode: DriveMode::PushPull,
    })
    .unwrap();

    // 상태 변화 있을 때만 OLED/사운드 갱신
    let mut was_pressed = false;

    loop {
        let pressed = boot.is_low();

        if pressed && !was_pressed {
            // ====== 눌림 시작: 소리 ON + OLED 표시 ======
            ch0.set_duty(50).unwrap(); // 50% 듀티로 소리

            display.clear(BinaryColor::Off).unwrap();
            Text::new("SOUND ON", Point::new(0, 16), style)
                .draw(&mut display)
                .unwrap();
            display.flush().unwrap();
        } else if !pressed && was_pressed {
            // ====== 뗌: 소리 OFF + OLED 지움(또는 OFF 표시) ======
            ch0.set_duty(0).unwrap(); // 소리 끔

            display.clear(BinaryColor::Off).unwrap();
            // OFF도 표시하고 싶으면 아래 주석 해제
            // Text::new("SOUND OFF", Point::new(0, 16), style)
            //     .draw(&mut display)
            //     .unwrap();
            display.flush().unwrap();
        }

        was_pressed = pressed;

        // 디바운스
        delay.delay_millis(15u32);
    }
}
