#![no_std]
#![no_main]

use embedded_graphics::{
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use fugit::{ExtU32, RateExtU32};
use panic_halt as _;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::pac;
use rp_pico::hal::Clock;

// The display driver:
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
    let movement_sensor_pin = pins.gpio9.into_pull_down_input();
    let mut movement_led_pin = pins
        .gpio21
        .into_push_pull_output_in_state(hal::gpio::PinState::Low);
    let mut buzzer = pins.gpio20.into_push_pull_output();

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_9X18_BOLD)
        .text_color(BinaryColor::On)
        .build();
    loop {
        Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        display.flush().unwrap();
        delay.delay_ms(200);
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
    }
}
