#![no_std]
#![no_main]

// The macro for our start-up function
use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_halt as _;
use rp_pico::entry;
use rp_pico::hal::pac;
// A shorter alias for the Hardware Abstraction Layer, which provides
// higher-level drivers.
use rp_pico::hal;
#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();

    // Note - we don't do any clock set-up in this example. The RP2040 will run
    // at it's default clock speed.

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Our LED output
    let mut led_pin = pins.gpio0.into_push_pull_output();

    // Our button input
    let button_pin = pins.gpio15.into_pull_up_input();

    // Run forever, setting the LED according to the button
    loop {
        if button_pin.is_low().unwrap() {
            led_pin.set_high().unwrap();
        } else {
            led_pin.set_low().unwrap();
        }
    }
}
