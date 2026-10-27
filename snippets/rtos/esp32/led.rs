// (개념 예시) esp-idf-svc 쪽은 std 기반이 많아서
// no_std esp-hal 예제와는 프로젝트 구성이 달라질 수 있어.
// 하지만 "RTOS식 구조"는 이게 정답.

use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::sys::*;

extern "C" fn led_task(arg: *mut core::ffi::c_void) {
    unsafe {
        let pin = arg as *mut PinDriver<'static, Gpio2, Output>;
        loop {
            (*pin).toggle().ok();
            vTaskDelay(pdMS_TO_TICKS(1000)); // ✅ RTOS delay (CPU 양보)
        }
    }
}

fn main() {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let mut led = PinDriver::output(peripherals.pins.gpio2).unwrap();

    // task에 넘길 수 있도록 'static'으로 만들거나 Box/Leak 등의 패턴을 씀
    let led_ptr: *mut _ = Box::into_raw(Box::new(led));

    unsafe {
        xTaskCreate(
            Some(led_task),
            b"led_task\0".as_ptr() as _,
            2048,
            led_ptr as _,
            5,
            core::ptr::null_mut(),
        );
    }

    // main은 보통 idle처럼 대기
    loop {
        unsafe {
            vTaskDelay(pdMS_TO_TICKS(10_000));
        }
    }
}
