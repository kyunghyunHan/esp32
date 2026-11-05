#![no_std]
#![no_main]

use core::panic::PanicInfo;

use esp_hal::{
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    main,
    uart::{Config as UartConfig, Uart},
};

esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

/*
 * ESP32-WROOM-32
 *
 * GPIO0  : BOOT button (Active LOW)
 * GPIO2  : Onboard LED (assumed Active HIGH)
 *
 * GPIO16 : UART1 RX <- STM32 PA9
 * GPIO17 : UART1 TX -> STM32 PA10
 *
 * UART: 9600bps, 8N1
 *
 * RX '1' -> LED ON
 * RX '0' -> LED OFF
 *
 * Button pressed  -> TX '1'
 * Button released -> TX '0'
 */

#[main]
fn main() -> ! {
    // 1. Initialize ESP32
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let mut delay = Delay::new();

    // 2. GPIO2 LED
    // Initial output = LOW (OFF for Active-High LED)
    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    // 3. GPIO0 BOOT button
    let button = Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    );

    // 4. UART1 configuration
    let config = UartConfig::default().with_baudrate(9600);

    let mut uart = Uart::new(peripherals.UART1, config)
        .unwrap()
        .with_rx(peripherals.GPIO25)
        .with_tx(peripherals.GPIO26);

    // 5. Initial button state
    let mut previous = button.is_low();

    // Allow STM32 UART initialization
    delay.delay_millis(300);

    let initial = if previous { b'1' } else { b'0' };
    let _ = uart.write(&[initial]);

    let mut rx_buf = [0u8; 1];

    // 6. Main loop
    loop {
        // ==============================
        // ESP32 BOOT -> STM32 LD2
        // ==============================
        let current = button.is_low();

        if current != previous {
            // Debounce
            delay.delay_millis(20);

            let confirmed = button.is_low();

            if confirmed != previous {
                let tx_data = if confirmed { b'1' } else { b'0' };

                let _ = uart.write(&[tx_data]);

                previous = confirmed;
            }
        }

        // ==============================
        // STM32 B1 -> ESP32 LED
        // ==============================
        if uart.read_ready() {
            match uart.read(&mut rx_buf) {
                Ok(1) => match rx_buf[0] {
                    b'1' => {
                        led.set_high();
                    }
                    b'0' => {
                        led.set_low();
                    }
                    _ => {}
                },
                Ok(_) => {}
                Err(_) => {
                    // UART receive error
                    // Continue polling
                }
            }
        }

        delay.delay_millis(1);
    }
}
