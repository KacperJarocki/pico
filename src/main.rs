#![no_std]
#![no_main]

use core::fmt::Write;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_5X8, FONT_9X18_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use fugit::RateExtU32;
use panic_halt as _;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::pac;
use rp_pico::hal::Clock;

use ssd1306::{prelude::*, Ssd1306};
#[entry]
fn main() -> ! {
    let mut is_locked: bool = false;
    let core = pac::CorePeripherals::take().unwrap();
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let sio = hal::Sio::new(pac.SIO);
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    delay.delay_ms(300);

    let mut row_1_pin = pins
        .gpio9
        .into_push_pull_output_in_state(hal::gpio::PinState::Low);

    let mut row_2_pin = pins
        .gpio8
        .into_push_pull_output_in_state(hal::gpio::PinState::Low);

    let mut row_3_pin = pins
        .gpio7
        .into_push_pull_output_in_state(hal::gpio::PinState::Low);

    let mut row_4_pin = pins
        .gpio6
        .into_push_pull_output_in_state(hal::gpio::PinState::Low);

    let col_1_pin = pins.gpio5.into_pull_down_input();
    let col_2_pin = pins.gpio4.into_pull_down_input();
    let col_3_pin = pins.gpio3.into_pull_down_input();
    let col_4_pin = pins.gpio2.into_pull_down_input();

    let sda_pin = pins.gpio16.into_function::<hal::gpio::FunctionI2C>();
    let scl_pin = pins.gpio17.into_function::<hal::gpio::FunctionI2C>();
    let i2c = hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );
    let interface = ssd1306::I2CDisplayInterface::new(i2c);
    let mut led_pin_locked = pins
        .gpio26
        .into_push_pull_output_in_state(hal::gpio::PinState::Low);
    let mut led_pin_unlocked = pins
        .gpio27
        .into_push_pull_output_in_state(hal::gpio::PinState::High);
    let button_pin = pins.gpio22.into_pull_up_input();
    let movement_sensor_pin = pins.gpio15.into_pull_down_input();
    let mut movement_led_pin = pins
        .gpio21
        .into_push_pull_output_in_state(hal::gpio::PinState::Low);
    let mut buzzer = pins.gpio20.into_push_pull_output();

    let text_style_code = MonoTextStyleBuilder::new()
        .font(&FONT_9X18_BOLD)
        .text_color(BinaryColor::On)
        .build();
    let text_style_mess = MonoTextStyleBuilder::new()
        .font(&FONT_5X8)
        .text_color(BinaryColor::On)
        .build();
    let mut code = FmtBuf::new();
    let mut mess = FmtBuf::new();

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    loop {
        display.init().unwrap();
        display.clear(BinaryColor::Off).unwrap();
        row_1_pin.set_high().unwrap();
        delay.delay_ms(20);
        if col_1_pin.is_high().unwrap() {
            code.write_str("1").unwrap();
            mess.reset();
            mess.write_str("Wiadomosc kod za krotki").unwrap();
        }
        if col_2_pin.is_high().unwrap() {
            code.write_str("2").unwrap();
        }

        if col_3_pin.is_high().unwrap() {
            code.write_str("3").unwrap();
        }
        if col_4_pin.is_high().unwrap() {
            code.write_str("A").unwrap();
        }
        row_1_pin.set_low().unwrap();
        row_2_pin.set_high().unwrap();
        delay.delay_ms(20);
        if col_1_pin.is_high().unwrap() {
            code.write_str("4").unwrap();
        }
        if col_2_pin.is_high().unwrap() {
            code.write_str("5").unwrap();
        }

        if col_3_pin.is_high().unwrap() {
            code.write_str("6").unwrap();
        }
        if col_4_pin.is_high().unwrap() {
            code.write_str("B").unwrap();
        }
        row_2_pin.set_low().unwrap();

        row_3_pin.set_high().unwrap();
        delay.delay_ms(20);
        if col_1_pin.is_high().unwrap() {
            code.write_str("7").unwrap();
        }
        if col_2_pin.is_high().unwrap() {
            code.write_str("8").unwrap();
        }

        if col_3_pin.is_high().unwrap() {
            code.write_str("9").unwrap();
        }
        if col_4_pin.is_high().unwrap() {
            code.write_str("C").unwrap();
        }
        row_3_pin.set_low().unwrap();

        row_4_pin.set_high().unwrap();
        delay.delay_ms(20);
        if col_1_pin.is_high().unwrap() {
            code.write_str("*").unwrap();
        }
        if col_2_pin.is_high().unwrap() {
            code.write_str("0").unwrap();
        }

        if col_3_pin.is_high().unwrap() {
            code.write_str("#").unwrap();
            mess.reset();
            mess.write_str("kod zatwierdzony").unwrap();
        }
        if col_4_pin.is_high().unwrap() {
            code.write_str("D").unwrap();
        }
        row_4_pin.set_low().unwrap();

        Text::with_baseline(code.as_str(), Point::zero(), text_style_code, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        Text::with_baseline(
            mess.as_str(),
            Point::new(1, 20),
            text_style_mess,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();
        if code.ptr > 4 {
            Text::with_baseline(
                "wystarczajaca dlugosc",
                Point::new(1, 40),
                text_style_mess,
                Baseline::Top,
            )
            .draw(&mut display)
            .unwrap();
        }

        display.flush().unwrap();

        delay.delay_ms(100);
        if button_pin.is_low().unwrap() {
            match is_locked {
                true => {
                    is_locked = false;
                    led_pin_locked.set_low().unwrap();
                    led_pin_unlocked.set_high().unwrap();
                }
                false => {
                    is_locked = true;
                    led_pin_unlocked.set_low().unwrap();
                    led_pin_locked.set_high().unwrap();
                }
            }
        }
        if is_locked && movement_sensor_pin.is_high().unwrap() {
            movement_led_pin.set_high().unwrap();
            buzzer.set_high().unwrap();
        }
        if !is_locked {
            movement_led_pin.set_low().unwrap();
            buzzer.set_low().unwrap();
        }
        delay.delay_ms(20);
    }
}

struct FmtBuf {
    buf: [u8; 64],
    ptr: usize,
}

impl FmtBuf {
    fn new() -> Self {
        Self {
            buf: [0; 64],
            ptr: 0,
        }
    }

    fn reset(&mut self) {
        self.ptr = 0;
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[0..self.ptr]).unwrap()
    }
}

impl core::fmt::Write for FmtBuf {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let rest_len = self.buf.len() - self.ptr;
        let len = if rest_len < s.len() {
            rest_len
        } else {
            s.len()
        };
        self.buf[self.ptr..(self.ptr + len)].copy_from_slice(&s.as_bytes()[0..len]);
        self.ptr += len;
        Ok(())
    }
}
