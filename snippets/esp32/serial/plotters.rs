#![no_std]
#![no_main]

use esp_hal::{
    clock::CpuClock,
    analog::adc::{Adc, AdcConfig, Attenuation},
    main,
};

use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // ADC 설정
    let mut adc = Adc::new(peripherals.ADC1, AdcConfig::new());
    let mut pin = peripherals.GPIO34.into_analog(); // A0 느낌

    loop {
        let value: u16 = adc.read(&mut pin, Attenuation::_11dB);

        // 👉 핵심: 숫자만 출력
        println!("{}", value);
    }
}