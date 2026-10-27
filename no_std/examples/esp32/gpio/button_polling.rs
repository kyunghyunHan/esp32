#![no_std]
#![no_main]

use esp_hal::{
    clock::CpuClock,
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

    // ✅ Arduino: LED_PIN = 21
    let mut led = Output::new(peripherals.GPIO21, Level::Low, OutputConfig::default());

    // ✅ Arduino: BUTTON_PIN = 18
    let button = Input::new(
        peripherals.GPIO18,
        InputConfig::default().with_pull(Pull::Up),
    );

    loop {
        // Arduino: digitalRead()
        if button.is_low() {
            // 버튼 눌림
            led.set_high(); // Arduino: digitalWrite(HIGH)
        } else {
            // 버튼 안눌림
            led.set_low(); // Arduino: digitalWrite(LOW)
        }
    }
}
