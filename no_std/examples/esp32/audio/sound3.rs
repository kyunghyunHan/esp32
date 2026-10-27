#![no_std]
#![no_main]

use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{DriveMode, Input, InputConfig, Pull},
    ledc::{
        LSGlobalClkSource, Ledc, LowSpeed,
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
    },
    main,
    time::Rate,
};

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

    // ✅ BOOT 버튼 (GPIO0, 눌리면 LOW)
    let boot = Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    );

    // ✅ LEDC(PWM) 초기화
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    // PWM 타이머: 2kHz, 10-bit 해상도
    let mut timer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    timer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty10Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(2_000),
        })
        .unwrap();

    // PWM 채널: Channel0 -> GPIO25
    let mut ch0 = ledc.channel(channel::Number::Channel0, peripherals.GPIO25);
    ch0.configure(channel::config::Config {
        timer: &timer0,
        duty_pct: 50,
        drive_mode: DriveMode::PushPull,
    })
    .unwrap();

    loop {
        if boot.is_low() {
            // 눌렀을 때: 소리 ON (50% 듀티)
            ch0.set_duty(50).unwrap();
        } else {
            // 떼면: 소리 OFF
            ch0.set_duty(0).unwrap();
        }

        // 버튼 채터링/CPU 점유 줄이기
        delay.delay_millis(10u32);
    }
}
