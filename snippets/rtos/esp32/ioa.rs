use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::prelude::*;
use esp_idf_sys as _;

use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() {
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let ir = PinDriver::input(pins.gpio4).unwrap();
    let mut buzzer = PinDriver::output(pins.gpio25).unwrap();

    let shared = Arc::new(Mutex::new(false));
    let shared_ir = shared.clone();

    // 🔹 Task 1: IR 읽기
    thread::spawn(move || {
        loop {
            let detected = ir.is_low();
            *shared_ir.lock().unwrap() = detected;
            thread::sleep(Duration::from_millis(10));
        }
    });

    // 🔹 Task 2: Buzzer 제어
    loop {
        let state = *shared.lock().unwrap();
        if state {
            buzzer.set_high().unwrap();
        } else {
            buzzer.set_low().unwrap();
        }
        thread::sleep(Duration::from_millis(10));
    }
}