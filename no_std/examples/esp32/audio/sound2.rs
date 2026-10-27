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

    // ✅ BOOT 버튼: GPIO0 (눌리면 LOW)
    let boot = Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    );

    // 🔔 BUZZER: GPIO25 출력
    let mut buzzer = Output::new(peripherals.GPIO25, Level::Low, OutputConfig::default());

    let mut was_pressed = false;

    loop {
        let pressed = boot.is_low();

        // "눌림 이벤트"에서만 1번 삑
        if pressed && !was_pressed {
            buzzer.set_high();
            delay.delay_millis(120u32);
            buzzer.set_low();
        }

        was_pressed = pressed;

        // 디바운스(채터링 방지) 약간
        delay.delay_millis(20u32);
    }
}
