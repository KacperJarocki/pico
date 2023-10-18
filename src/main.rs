#![no_std]
#![no_main]

// The macro for our start-up function
use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_halt as _;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::pac;
use rp_pico::hal::Clock;
#[entry]
fn main() -> ! {
    let mut is_locked: bool = false;
    let core = pac::CorePeripherals::take().unwrap();
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // The single-cycle I/O block controls our GPIO pins
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

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Our LED output
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
    loop {
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
        delay.delay_ms(200);
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
