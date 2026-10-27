#![no_std]
#![no_main]

use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    main,
    time::{Duration, Instant},
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

    // GPIO2 = LED
    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    // GPIO4 = Button
    // Pull::Up 이므로:
    // 기본 상태 = HIGH
    // 버튼 누름 = LOW
    let button = Input::new(
        peripherals.GPIO4,
        InputConfig::default().with_pull(Pull::Up),
    );

    loop {
        if button.is_low() {
            // 버튼 바운스 방지
            delay_ms(20);

            // 20ms 뒤에도 여전히 LOW면 진짜 눌린 것
            if button.is_low() {
                led.set_high();
            }
        } else {
            led.set_low();
        }
    }
}

fn delay_ms(ms: u64) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(ms) {}
}
