#![no_std]
#![no_main]

use bsp::entry;
use bsp::hal;
use defmt_rtt as _;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use hal::pac;
use panic_halt as _;

use rp_pico::hal::gpio::FunctionUart;
use rp_pico::hal::uart::*;
use rp_pico::hal::Clock;

use rp_pico as bsp;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure GPIO25 as output
    let mut led_pin = pins.gpio25.into_push_pull_output();

    // Config UART
    let uart_pins = (
        pins.gpio0.into_function::<FunctionUart>(),
        pins.gpio1.into_function::<FunctionUart>(),
    );

    let uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(UartConfig::default(), clocks.peripheral_clock.freq())
        .unwrap();

    uart.write_full_blocking(b"Hello, World!\r\n");

    loop {
        uart.write_full_blocking(b"Hello!\r\n");
        led_pin.set_high().unwrap();
        timer.delay_ms(500);
        led_pin.set_low().unwrap();
        timer.delay_ms(500);
    }
}
