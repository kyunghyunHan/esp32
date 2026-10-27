use esp_idf_hal::delay::FreeRtos;
use esp_idf_sys::{self as _};
use esp_idf_sys::xTaskCreatePinnedToCore;
use std::ffi::CString;

unsafe extern "C" fn lamp_task(_: *mut core::ffi::c_void) {
    loop {
        println!("Lamp ON");
        FreeRtos::delay_ms(100);

        println!("Lamp OFF");
        FreeRtos::delay_ms(100);
    }
}

unsafe extern "C" fn motor_task(_: *mut core::ffi::c_void) {
    loop {
        println!("Motor ON");
        FreeRtos::delay_ms(100);

        println!("Motor OFF");
        FreeRtos::delay_ms(100);
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();

    unsafe {
        xTaskCreatePinnedToCore(
            Some(lamp_task),
            CString::new("Lamp Task").unwrap().as_ptr(),
            4096,
            std::ptr::null_mut(),
            10,
            std::ptr::null_mut(),
            1,
        );
    }

    unsafe {
        xTaskCreatePinnedToCore(
            Some(motor_task),
            CString::new("Motor Task").unwrap().as_ptr(),
            4096,
            std::ptr::null_mut(),
            9,
            std::ptr::null_mut(),
            1,
        );
    }

    loop {
        println!("Main Running");
        FreeRtos::delay_ms(500);
    }
}