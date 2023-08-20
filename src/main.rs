use anyhow::Result;
use embedded_graphics::{
    draw_target,
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_hal::blocking::delay::DelayMs;
use esp_idf_hal::{
    delay::FreeRtos,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    prelude::*,
};
use shared_bus::BusManagerSimple;
use shtcx::{self, PowerMode as shtPowerMode};

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let sda = peripherals.pins.gpio10;
    let scl = peripherals.pins.gpio8;

    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config)?;

    let bus = BusManagerSimple::new(i2c);

    let proxy_1 = bus.acquire_i2c();
    let proxy_2 = bus.acquire_i2c();

    let mut sht = shtcx::shtc3(proxy_1);
    let device_id = sht.device_identifier().unwrap();

    let mut interface = I2CDisplayInterface::new(proxy_2);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    loop {
        sht.start_measurement(shtPowerMode::NormalMode).unwrap();
        FreeRtos.delay_ms(500u32);
        let measurement = sht.get_measurement_result().unwrap();
        let temperature = measurement.temperature.as_degrees_celsius() * 1.8 + 32.0;
        let temperature_text = String::from(format!("Temp: {:.1} °F", temperature,));
        let humidity_text = String::from(format!(
            "Humidity: %{:.1}",
            measurement.humidity.as_percent()
        ));

        display.clear_buffer();
        display.flush().unwrap();
        Text::with_baseline(
            "I love you, Mercedes!",
            Point::zero(),
            text_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        Text::with_baseline(
            &temperature_text,
            Point::new(0, 16),
            text_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        Text::with_baseline(&humidity_text, Point::new(0, 48), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        display.flush().unwrap();
    }
}
