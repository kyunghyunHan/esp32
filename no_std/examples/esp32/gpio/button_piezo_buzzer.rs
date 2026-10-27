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

    let button = Input::new(
        peripherals.GPIO16,
        InputConfig::default().with_pull(Pull::Up),
    );

    let mut buzzer = Output::new(peripherals.GPIO21, Level::Low, OutputConfig::default());

    loop {
        let button_pressed = button.is_low();
        if button_pressed {
            buzzer.set_high();
        } else {
            buzzer.set_low();
        }

        delay.delay_millis(50);
    }
}
