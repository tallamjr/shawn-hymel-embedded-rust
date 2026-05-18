#![no_std]
#![no_main]

// We need to write our own panic handler
use core::panic::PanicInfo;

// Alias our HAL
use rp2040_hal as hal;

// Import traits for embedded abstractions
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

// Custom panic handler: just loop forever
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Copy bootloader from rp2040-boot2 into BOOT2 section of memory
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

// Set external crystal frequency
const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

// Generic LED struct — identical to the rp235x version. The whole point of
// the lecture is that this code is portable across MCU families because it
// is written against the embedded-hal `OutputPin` trait, not against any
// particular HAL's concrete pin type.
struct Led<P> {
    pin: P,
    active_high: bool,
}

impl<P> Led<P>
where
    P: OutputPin,
{
    fn new(pin: P, active_high: bool) -> Self {
        Self { pin, active_high }
    }

    fn on(&mut self) {
        if self.active_high {
            self.pin.set_high().unwrap();
        } else {
            self.pin.set_low().unwrap();
        }
    }

    fn off(&mut self) {
        if self.active_high {
            self.pin.set_low().unwrap();
        } else {
            self.pin.set_high().unwrap();
        }
    }
}

// Main entrypoint (custom defined for embedded targets)
#[hal::entry]
fn main() -> ! {
    // Get ownership of hardware peripherals
    let mut pac = hal::pac::Peripherals::take().unwrap();

    // Set up the watchdog and clocks
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Single-cycle I/O block (fast GPIO)
    let sio = hal::Sio::new(pac.SIO);

    // Split off ownership of Peripherals struct, set pins to default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // GPIO25 = Pi Pico's onboard LED (the course uses GPIO15 for a
    // breadboard LED; the Faultier carrier has no external LED but exposes
    // the Pico's onboard one).
    let led_pin = pins.gpio25.into_push_pull_output();

    // Wrap the concrete pin in the generic Led<P>. Type inference fills in
    // P = Pin<Gpio25, FunctionSio<SioOutput>, PullDown>. No annotation needed.
    let mut led = Led::new(led_pin, true);

    // Move ownership of TIMER peripheral to create Timer struct
    // (rp2040-hal: single TIMER; rp235x-hal: TIMER0 / TIMER1)
    let mut timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // Blink loop
    loop {
        led.on();
        timer.delay_ms(500);
        led.off();
        timer.delay_ms(500);
    }
}
