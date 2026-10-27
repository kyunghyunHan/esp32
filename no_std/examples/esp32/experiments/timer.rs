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

use core::cell::RefCell;
use core::fmt::Write;
use embedded_hal::i2c::I2c as _;
use embedded_hal::pwm::SetDutyCycle;
use embedded_hal_bus::i2c::RefCellDevice;
use heapless::String;
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

use ds323x::{DateTimeAccess, Ds323x, Timelike};
use rtcc::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

// ⚠️ esp-hal(no_std)에서는 이거 빼는게 안전
esp_bootloader_esp_idf::esp_app_desc!();

// ====== 알람 설정 ======
const ALARM_HH: u32 = 7;
const ALARM_MM: u32 = 30;

// 부저 주파수(패시브 부저용)
const BUZZ_HZ: u32 = 2000;

// OLED에 한 줄 출력 헬퍼
fn oled_line<D: DrawTarget<Color = BinaryColor>>(
    d: &mut D,
    style: MonoTextStyle<'_, BinaryColor>,
    y: i32,
    msg: &str,
) {
    Text::new(msg, Point::new(0, y), style).draw(d).ok();
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);
    let delay = Delay::new();

    // BOOT 버튼(GPIO0)
    let stop_btn = Input::new(p.GPIO0, InputConfig::default().with_pull(Pull::Up));

    // I2C
    let i2c = I2c::new(
        p.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(p.GPIO21)
    .with_scl(p.GPIO22);

    let i2c = RefCell::new(i2c);

    // OLED
    let interface = I2CDisplayInterface::new(RefCellDevice::new(&i2c));
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().ok();
    display.clear_buffer();

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    // ====== [DEBUG 1] RTC 주소(0x68) 핑 테스트 ======
    // (빈 write 성공하면 응답 있다고 판단)
    let mut ping_dev = RefCellDevice::new(&i2c);
    let rtc_ping_ok = ping_dev.write(0x68, &[]).is_ok();

    display.clear_buffer();
    oled_line(&mut display, style, 10, "I2C PING:");
    if rtc_ping_ok {
        oled_line(&mut display, style, 24, "RTC 0x68: OK");
    } else {
        oled_line(&mut display, style, 24, "RTC 0x68: FAIL");
        oled_line(&mut display, style, 38, "CHECK WIRING!");
    }
    display.flush().ok();
    delay.delay_millis(2000);

    // DS3231 드라이버
    let mut rtc = Ds323x::new_ds3231(RefCellDevice::new(&i2c));

    // ====== [DEBUG 2] 부팅 후 3초: BOOT 누르면 시간 세팅 ======
    display.clear_buffer();
    oled_line(&mut display, style, 16, "Press BOOT");
    oled_line(&mut display, style, 28, "within 3s");
    display.flush().ok();

    let mut pressed = false;
    for _ in 0..30 {
        if stop_btn.is_low() {
            pressed = true;
            break;
        }
        delay.delay_millis(100);
    }

    if pressed {
        // 원하는 시간으로 수정
        let date = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap();
        let time = NaiveTime::from_hms_opt(15, 0, 0).unwrap();
        let now = NaiveDateTime::new(date, time);

        let set_ok = rtc.set_datetime(&now).is_ok();

        display.clear_buffer();
        oled_line(&mut display, style, 10, "SET TIME RESULT:");
        if set_ok {
            oled_line(&mut display, style, 24, "SET: OK");
        } else {
            oled_line(&mut display, style, 24, "SET: FAIL");
        }

        // 세팅 후 즉시 읽어서 표시
        match rtc.datetime() {
            Ok(dt) => {
                let mut buf: String<32> = String::new();
                let _ = write!(
                    buf,
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    dt.year(),
                    dt.month(),
                    dt.day(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                );
                oled_line(&mut display, style, 40, &buf);
            }
            Err(_) => {
                oled_line(&mut display, style, 40, "READ FAIL");
            }
        }

        display.flush().ok();
        delay.delay_millis(2000);
    }

    // ====== 부저 PWM ======
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
    let mut alarming = false;
    let mut tick_ms: u32 = 0;

    loop {
        // 알람 끄기
        if stop_btn.is_low() && alarming {
            alarming = false;
            buzzer.set_duty_cycle_percent(0).ok();
        }

        delay.delay_millis(100);
        tick_ms += 100;

        if tick_ms >= 1000 {
            tick_ms = 0;

            // ====== [DEBUG 3] 매초 RTC 읽기 + 그대로 표시 ======
            match rtc.datetime() {
                Ok(dt) => {
                    let h = dt.hour();
                    let m = dt.minute();
                    let s = dt.second();

                    // 알람 체크
                    if h == ALARM_HH && m == ALARM_MM && s == 0 {
                        alarming = true;
                    }

                    let mut buf: String<16> = String::new();
                    let _ = write!(buf, "{:02}:{:02}:{:02}", h, m, s);

                    let mut date_buf: String<16> = String::new();
                    let _ = write!(
                        date_buf,
                        "{:04}-{:02}-{:02}",
                        dt.year(),
                        dt.month(),
                        dt.day()
                    );

                    display.clear_buffer();
                    oled_line(&mut display, style, 10, "ALARM CLOCK");
                    oled_line(&mut display, style, 24, &date_buf);
                    oled_line(&mut display, style, 40, &buf);

                    // 디버그 줄: 알람/버튼 상태
                    let mut dbg: String<32> = String::new();
                    let _ = write!(
                        dbg,
                        "A:{} B:{}",
                        if alarming { 1 } else { 0 },
                        if stop_btn.is_low() { 1 } else { 0 },
                    );
                    oled_line(&mut display, style, 56, &dbg);

                    display.flush().ok();
                }
                Err(_) => {
                    display.clear_buffer();
                    oled_line(&mut display, style, 10, "RTC READ FAIL");
                    oled_line(&mut display, style, 24, "ADDR 0x68?");
                    display.flush().ok();
                }
            }
        }

        // 알람 울림
        if alarming {
            buzzer.set_duty_cycle_percent(50).ok();
        } else {
            buzzer.set_duty_cycle_percent(0).ok();
        }
    }
}
