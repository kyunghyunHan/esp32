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

use embedded_hal::pwm::SetDutyCycle;
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

// DS3231 (사용 시 Cargo.toml에 ds323x/rtcc 추가 필요)

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

esp_bootloader_esp_idf::esp_app_desc!();

// 부저 주파수(패시브 부저용)
const BUZZ_HZ: u32 = 2000;

#[main]
fn main() -> ! {
    // 1) 기본 init
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);
    let delay = Delay::new();

    // 2) 버튼(BOOT=GPIO0) - 눌리면 LOW
    let stop_btn = Input::new(p.GPIO0, InputConfig::default().with_pull(Pull::Up));

    // 3) I2C (OLED + DS3231 공유)
    // ⚠️ 핀은 보드에 맞게 바꿔도 됨
    let i2c = I2c::new(
        p.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    // 예: SDA=GPIO21, SCL=GPIO22 (일반 ESP32 DevKit)
    .with_sda(p.GPIO21)
    .with_scl(p.GPIO22);

    // 4) OLED
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().ok();
    display.clear_buffer();

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    // 5) DS3231 (OLED랑 같은 I2C 버스 위)
    // 주의: 위에서 i2c를 OLED가 먹었으니 "interface" 안쪽 i2c를 공유해야 함.
    // 가장 쉬운 방법은: OLED/RTC를 같은 I2C로 만들되, "shared-bus"를 쓰는 방식.
    //
    // 여기서는 개념을 보여주기 위해 아래에 "shared-bus" 방식으로 바꾸는 걸 추천할게.
    //
    // ---- 아래 'shared-bus' 버전은 바로 밑에 따로 제공 ----

    // 6) 부저 PWM(LEDC)
    let mut ledc = Ledc::new(p.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut timer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    timer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty10Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(BUZZ_HZ),
        })
        .unwrap();

    let mut buzzer = ledc.channel(channel::Number::Channel0, p.GPIO4);
    buzzer
        .configure(channel::config::Config {
            timer: &timer0,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    // 알람 상태
    let mut alarming = false;

    loop {
        // (A) 여기서 RTC 시간을 읽어야 하는데,
        // OLED와 RTC가 i2c를 공유해야 해서 shared-bus 버전으로 구성하는게 정석.
        //
        // (B) 일단 디스플레이는 예시로 텍스트만 보여줌
        display.clear_buffer();
        Text::new("ALARM CLOCK", Point::new(0, 10), style)
            .draw(&mut display)
            .ok();

        // TODO: RTC에서 읽은 시각 표시
        // Text::new(&format!("07:30:12"), Point::new(0, 30), style)...

        display.flush().ok();

        // 버튼 누르면 알람 끄기
        if stop_btn.is_low() && alarming {
            alarming = false;
            buzzer.set_duty_cycle_percent(0).ok();
        }

        delay.delay_millis(200);
    }
}
