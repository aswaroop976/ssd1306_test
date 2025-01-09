//! Draw Ferris the Rust mascot on an SSD1306 display
//!
//! For example, to run on an STM32F411 Nucleo
//! dev board, run the following:
//!
//! ```bash
//! cargo run --features stm32f411--release --example ssd1306-image
//! ```
//!
//! Note that `--release` is required to fix link errors for smaller devices.

#![allow(clippy::empty_loop)]
#![no_std]
#![no_main]

use panic_semihosting as _;
use stm32f4xx_hal as hal;

use cortex_m_rt::ExceptionFrame;
use cortex_m_rt::{entry, exception};
use embedded_graphics::{image::Image, image::ImageRaw, pixelcolor::BinaryColor, prelude::*};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use crate::hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(_cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(100.MHz()).freeze();

        // Set up I2C - SCL is PB6 and SDA is PB7; they are set to Alternate Function 4
        // as per the STM32F446xC/E datasheet page 60. Pin assignment as per the Nucleo-F446 board.
        let gpiob = dp.GPIOB.split();
        let scl = gpiob.pb6.internal_pull_up(true);
        let sda = gpiob.pb7.internal_pull_up(true);
        // let i2c = I2c::new(dp.I2C1, (scl, sda), 400.kHz(), &clocks);
        // or
        let i2c = dp.I2C1.i2c((scl, sda), 400.kHz(), &clocks);

        // There's a button on PC13. On the Nucleo board, it's pulled up by a 4.7kOhm resistor
        // and therefore is active LOW. There's even a 100nF capacitor for debouncing - nice for us
        // since otherwise we'd have to debounce in software.
        // let gpioc = dp.GPIOC.split();
        // let btn = gpioc.pc13.into_pull_down_input();

        // Set up the display
        let interface = I2CDisplayInterface::new(i2c);
        let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        disp.init().unwrap();
        disp.flush().unwrap();

        // Draw a single pixel at (10, 10)
        Pixel(Point::new(10, 10), BinaryColor::On)
            .draw(&mut disp)
            .unwrap();

        // Flush the display to apply the changes
        disp.flush().unwrap();
    }

    loop {}
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
