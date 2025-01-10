#![allow(clippy::empty_loop)]
#![no_std]
#![no_main]

use cortex_m_rt::ExceptionFrame;
use cortex_m_rt::{entry, exception};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use panic_semihosting as _;
use rtt_target::{rprintln, rtt_init_print};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use stm32f4xx_hal::{self as hal, pac};

use crate::hal::{
    gpio::{gpiob::PB0, gpiob::PB1, Input, Output, PushPull},
    pac::Interrupt,
    prelude::*,
};
#[entry]
fn main() -> ! {
    rtt_init_print!();
    if let (Some(dp), Some(_cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(100.MHz()).freeze();

        // Set up I2C - SCL is PB6 and SDA is PB7; they are set to Alternate Function 4
        let gpiob = dp.GPIOB.split();
        let scl = gpiob.pb6.internal_pull_up(true);
        let sda = gpiob.pb7.internal_pull_up(true);
        // Configure PB0 as an output pin (connected to one side of the button)
        let mut output_pin: PB0<Output<PushPull>> = gpiob.pb0.into_push_pull_output();
        // Configure PB1 as an input pin as pull down input
        let input_pin: PB1<Input> = gpiob.pb1.into_pull_down_input();

        output_pin.set_high();
        let i2c = dp.I2C1.i2c((scl, sda), 400.kHz(), &clocks);

        // Set up the display
        let interface = I2CDisplayInterface::new(i2c);
        let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        disp.init().unwrap();
        disp.flush().unwrap();

        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            if input_pin.is_high() {
                if i < 127 {
                    i += 1;
                } else if j < 63 {
                    i = 0;
                    j += 1;
                }

                Pixel(Point::new(i.into(), j), BinaryColor::On)
                    .draw(&mut disp)
                    .unwrap();

                // Flush the display to apply the changes
                disp.flush().unwrap();
                rprintln!("pressed");
                rprintln!("i is: {}", i);
            } else {
                rprintln!("not pressed");
            }
        }
    }

    loop {}
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
