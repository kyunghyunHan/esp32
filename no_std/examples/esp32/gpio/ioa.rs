#![no_std]
#![no_main]

use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
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

    // ✅ IR 센서 (GPIO4)
    let ir = Input::new(
        peripherals.GPIO4,
        InputConfig::default().with_pull(Pull::Up),
    );

    // ✅ Active buzzer (GPIO25)
    let mut buzzer = Output::new(peripherals.GPIO25, Level::Low, OutputConfig::default());

    loop {
        if ir.is_low() {
            // 물체 감지 → 소리 ON
            buzzer.set_high();
        } else {
            // 물체 없음 → 소리 OFF
            buzzer.set_low();
        }

        delay.delay_millis(10u32);
    }
}
