#![no_std]
#![no_main]

use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main,
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

    // 🔔 GPIO25를 출력으로 설정
    let mut buzzer = Output::new(peripherals.GPIO25, Level::Low, OutputConfig::default());

    loop {
        buzzer.set_high(); // 삐
        delay.delay_millis(200u32);

        buzzer.set_low(); // 무음
        delay.delay_millis(800u32);
    }
}
