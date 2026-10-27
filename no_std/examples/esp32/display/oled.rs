#![no_std]
#![no_main]

use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Pull},
    i2c::master::{Config, I2c},
    main,
    time::Rate,
};

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle, RoundedRectangle},
};

use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

esp_bootloader_esp_idf::esp_app_desc!();

fn draw_normal<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    display.clear(BinaryColor::Off)?;

    let fill = PrimitiveStyle::with_fill(BinaryColor::On);

    // 왼쪽 눈
    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(18, 18), Size::new(34, 28)),
        Size::new(8, 8),
    )
    .into_styled(fill)
    .draw(display)?;

    // 오른쪽 눈
    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(76, 18), Size::new(34, 28)),
        Size::new(8, 8),
    )
    .into_styled(fill)
    .draw(display)?;

    Ok(())
}

fn draw_happy<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    display.clear(BinaryColor::Off)?;

    let style = PrimitiveStyle::with_stroke(BinaryColor::On, 4);

    // 왼쪽 웃는 눈
    Line::new(Point::new(18, 36), Point::new(34, 24))
        .into_styled(style)
        .draw(display)?;

    Line::new(Point::new(34, 24), Point::new(50, 36))
        .into_styled(style)
        .draw(display)?;

    // 오른쪽 웃는 눈
    Line::new(Point::new(76, 36), Point::new(92, 24))
        .into_styled(style)
        .draw(display)?;

    Line::new(Point::new(92, 24), Point::new(108, 36))
        .into_styled(style)
        .draw(display)?;

    Ok(())
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());

    let peripherals = esp_hal::init(config);

    // BOOT 버튼 GPIO0
    let boot = Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    );

    // I2C
    let i2c = I2c::new(
        peripherals.I2C0,
        Config::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO21)
    .with_scl(peripherals.GPIO22);

    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();

    // 처음에는 평범한 얼굴
    draw_normal(&mut display).unwrap();
    display.flush().unwrap();

    let mut was_pressed = false;

    loop {
        let pressed = boot.is_low();

        // 버튼 눌렀을 때
        if pressed && !was_pressed {
            draw_happy(&mut display).unwrap();
            display.flush().unwrap();
        }

        // 버튼 뗐을 때
        if !pressed && was_pressed {
            draw_normal(&mut display).unwrap();
            display.flush().unwrap();
        }

        was_pressed = pressed;
    }
}
