#![no_std]
#![no_main]

use core::fmt::Write;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_5X7, FONT_9X18_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    prelude::_embedded_hal_blocking_delay_DelayMs,
};
use fugit::RateExtU32;
use panic_halt as _;
use rp_pico::hal;
use rp_pico::hal::pac;
use rp_pico::hal::Clock;
use rp_pico::{entry, hal::gpio::Error};

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
        .font(&FONT_5X7)
        .text_color(BinaryColor::On)
        .build();
    let mut code = FmtBuf::new();
    let mut mess = FmtBuf::new();
    let mut unlocking = FmtBuf::new();

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    loop {
        display.clear(BinaryColor::Off).unwrap();

        Text::with_baseline(
            "Popraw kod wciskajac *\nZatwierdz kod wciskajac #",
            Point::new(1, 50),
            text_style_mess,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        match is_locked {
            false => {
                delay.delay_ms(30);
                led_pin_locked.set_low().unwrap();
                led_pin_unlocked.set_high().unwrap();
                get_alarm_locked(
                    &mut row_1_pin,
                    &mut row_2_pin,
                    &mut row_3_pin,
                    &mut row_4_pin,
                    &col_1_pin,
                    &col_2_pin,
                    &col_3_pin,
                    &col_4_pin,
                    &mut delay,
                    &mut code,
                    &mut mess,
                    &mut is_locked,
                );
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
            }
            true => {
                delay.delay_ms(30);
                led_pin_unlocked.set_low().unwrap();
                led_pin_locked.set_high().unwrap();
                get_alarm_unlocked(
                    &mut row_1_pin,
                    &mut row_2_pin,
                    &mut row_3_pin,
                    &mut row_4_pin,
                    &col_1_pin,
                    &col_2_pin,
                    &col_3_pin,
                    &col_4_pin,
                    &mut delay,
                    &mut unlocking,
                    &mut code,
                    &mut mess,
                    &mut is_locked,
                );
                Text::with_baseline(
                    unlocking.as_str(),
                    Point::zero(),
                    text_style_code,
                    Baseline::Top,
                )
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
            }
        }
        display.flush().unwrap();

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
fn get_alarm_unlocked<'a>(
    row_1_pin: &'a mut dyn OutputPin<Error = Error>,
    row_2_pin: &'a mut dyn OutputPin<Error = Error>,
    row_3_pin: &'a mut dyn OutputPin<Error = Error>,
    row_4_pin: &'a mut dyn OutputPin<Error = Error>,
    col_1_pin: &'a dyn InputPin<Error = Error>,
    col_2_pin: &'a dyn InputPin<Error = Error>,
    col_3_pin: &'a dyn InputPin<Error = Error>,
    col_4_pin: &'a dyn InputPin<Error = Error>,
    delay: &'a mut dyn _embedded_hal_blocking_delay_DelayMs<u16>,
    unlocking: &mut FmtBuf,
    code: &mut FmtBuf,
    mess: &mut FmtBuf,
    is_locked: &mut bool,
) {
    let how_to_handle_code = get_code_from_keyboard(
        row_1_pin, row_2_pin, row_3_pin, row_4_pin, col_1_pin, col_2_pin, col_3_pin, col_4_pin,
        delay, unlocking,
    );
    match how_to_handle_code {
        0 => {
            if code.is_equal(&unlocking) {
                unlocking.reset();
                code.reset();
                *is_locked = false;
            } else {
                mess.reset();
                unlocking.reset();
                mess.write_str("kod nieprawidlowy").unwrap();
            }
        }
        1 => {
            mess.reset();
            mess.write_str("Wpisz nowy kod\nnastepnie zatwierdz #")
                .unwrap();
        }
        2 => {
            mess.reset();
            mess.write_str("Wpisuj dalej kod\naby poprawic wcisnij *")
                .unwrap();
        }
        3 => {
            mess.reset();
            mess.write_str("Wpisuj dalej kod\naby poprawic wcisnij *")
                .unwrap();
        }
        _ => {}
    }
}
fn get_alarm_locked<'a>(
    row_1_pin: &'a mut dyn OutputPin<Error = Error>,
    row_2_pin: &'a mut dyn OutputPin<Error = Error>,
    row_3_pin: &'a mut dyn OutputPin<Error = Error>,
    row_4_pin: &'a mut dyn OutputPin<Error = Error>,
    col_1_pin: &'a dyn InputPin<Error = Error>,
    col_2_pin: &'a dyn InputPin<Error = Error>,
    col_3_pin: &'a dyn InputPin<Error = Error>,
    col_4_pin: &'a dyn InputPin<Error = Error>,
    delay: &'a mut dyn _embedded_hal_blocking_delay_DelayMs<u16>,
    code: &mut FmtBuf,
    mess: &mut FmtBuf,
    is_locked: &mut bool,
) {
    let how_to_handle_code = get_code_from_keyboard(
        row_1_pin, row_2_pin, row_3_pin, row_4_pin, col_1_pin, col_2_pin, col_3_pin, col_4_pin,
        delay, code,
    );
    match how_to_handle_code {
        0 => {
            *is_locked = true;
            mess.reset();
            mess.write_str("alarm uzbrojono").unwrap();
        }
        1 => {
            mess.reset();
            mess.write_str("Wpisz nowy kod\nnastepnie zatwierdz #")
                .unwrap();
        }
        2 => {
            mess.reset();
            mess.write_str("Wpisuj dalej kod\naby poprawic wcisnij *")
                .unwrap();
        }
        3 => {
            mess.reset();
            mess.write_str("Kod ma wystarczajaca\ndlugosc aby poprawic\nwcisnij *")
                .unwrap();
        }
        _ => {}
    }
}
fn get_code_from_keyboard<'a>(
    row_1_pin: &'a mut dyn OutputPin<Error = Error>,
    row_2_pin: &'a mut dyn OutputPin<Error = Error>,
    row_3_pin: &'a mut dyn OutputPin<Error = Error>,
    row_4_pin: &'a mut dyn OutputPin<Error = Error>,
    col_1_pin: &'a dyn InputPin<Error = Error>,
    col_2_pin: &'a dyn InputPin<Error = Error>,
    col_3_pin: &'a dyn InputPin<Error = Error>,
    col_4_pin: &'a dyn InputPin<Error = Error>,
    delay: &'a mut dyn _embedded_hal_blocking_delay_DelayMs<u16>,
    code: &mut FmtBuf,
) -> u32 {
    let _key_pressed = get_key_pressed_on_keyboard(
        row_1_pin, row_2_pin, row_3_pin, row_4_pin, col_1_pin, col_2_pin, col_3_pin, col_4_pin,
        delay,
    );
    let is_code_too_short = code.ptr < 4;
    let is_code_too_long = code.ptr > 8;
    let is_code_proper = !is_code_too_long && !is_code_too_short;
    if _key_pressed == "#" && is_code_proper {
        return 0;
    } else if _key_pressed == "#" && is_code_too_long {
        return 8;
    } else if _key_pressed == "#" && is_code_too_short {
        return 9;
    } else if _key_pressed == "*" {
        code.reset();
        return 1;
    } else if !is_code_proper {
        code.write_str(_key_pressed).unwrap();
        return 2;
    } else {
        code.write_str(_key_pressed).unwrap();
        return 3;
    }
}
fn get_key_pressed_on_keyboard<'a>(
    row_1_pin: &'a mut dyn OutputPin<Error = Error>,
    row_2_pin: &'a mut dyn OutputPin<Error = Error>,
    row_3_pin: &'a mut dyn OutputPin<Error = Error>,
    row_4_pin: &'a mut dyn OutputPin<Error = Error>,
    col_1_pin: &'a dyn InputPin<Error = Error>,
    col_2_pin: &'a dyn InputPin<Error = Error>,
    col_3_pin: &'a dyn InputPin<Error = Error>,
    col_4_pin: &'a dyn InputPin<Error = Error>,
    delay: &'a mut dyn _embedded_hal_blocking_delay_DelayMs<u16>,
) -> &'a str {
    row_1_pin.set_high().unwrap();
    delay.delay_ms(20);
    let mut key = "";
    if col_1_pin.is_high().unwrap() {
        key = "1";
    }
    if col_2_pin.is_high().unwrap() {
        key = "2";
    }
    if col_3_pin.is_high().unwrap() {
        key = "3";
    }
    if col_4_pin.is_high().unwrap() {
        key = "A";
    }
    row_1_pin.set_low().unwrap();
    row_2_pin.set_high().unwrap();
    delay.delay_ms(20);
    if col_1_pin.is_high().unwrap() {
        key = "4";
    }
    if col_2_pin.is_high().unwrap() {
        key = "5";
    }

    if col_3_pin.is_high().unwrap() {
        key = "6";
    }
    if col_4_pin.is_high().unwrap() {
        key = "B";
    }
    row_2_pin.set_low().unwrap();

    row_3_pin.set_high().unwrap();
    delay.delay_ms(20);
    if col_1_pin.is_high().unwrap() {
        key = "7";
    }
    if col_2_pin.is_high().unwrap() {
        key = "8";
    }

    if col_3_pin.is_high().unwrap() {
        key = "9";
    }
    if col_4_pin.is_high().unwrap() {
        key = "C";
    }
    row_3_pin.set_low().unwrap();
    row_4_pin.set_high().unwrap();
    delay.delay_ms(20);
    if col_1_pin.is_high().unwrap() {
        key = "*";
    }
    if col_2_pin.is_high().unwrap() {
        key = "0";
    }

    if col_3_pin.is_high().unwrap() {
        key = "#";
    }
    if col_4_pin.is_high().unwrap() {
        key = "D";
    }
    row_4_pin.set_low().unwrap();
    key
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
    fn is_equal(&self, s: &FmtBuf) -> bool {
        if self.ptr != s.ptr {
            false
        } else {
            for n in 0..self.ptr {
                if self.buf[n] != s.buf[n] {
                    return false;
                }
            }
            true
        }
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
