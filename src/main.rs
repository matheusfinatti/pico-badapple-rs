#![no_std]
#![no_main]

use bsp::entry;
use bsp::hal;
use hal::pac;
use panic_halt as _;

use hal::gpio::*;
use hal::Clock;

use rp_pico as bsp;

// Mipidsi drivers
use mipidsi::*;

// Embedded HAL
use embedded_hal::digital::OutputPin;

// Embedded graphics to draw on the display
use embedded_graphics::draw_target::*;
use embedded_graphics::geometry::*;
use embedded_graphics::pixelcolor::*;
use embedded_graphics::primitives::*;
use embedded_graphics::Drawable;

// Embed the 'Hz' function.
use fugit::RateExtU32;

use rtt_target::{rprintln, rtt_init_print};

// Display specs
const WIDTH: i32 = 320;
const HEIGHT: i32 = 240;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    rprintln!("Starting");

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

    // Configure LED pin as output
    let mut led_pin = pins.gpio25.into_push_pull_output();

    // Configure SPI pins for display
    let spi_mosi: Pin<_, FunctionSpi, PullNone> = pins.gpio19.reconfigure();
    let spi_sclk: Pin<_, FunctionSpi, PullNone> = pins.gpio18.reconfigure();

    // Init SPI
    let spi = hal::Spi::<_, _, _, 8>::new(pac.SPI0, (spi_mosi, spi_sclk));
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        62_500_000.Hz(),
        &embedded_hal::spi::MODE_0,
    );

    let dc = pins.gpio16.into_push_pull_output();
    let cs = pins.gpio17.into_push_pull_output();

    // SPI interface
    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new(spi, cs, timer).unwrap();
    let di = display_interface_spi::SPIInterface::new(spi_device, dc);

    // Reset pin must be set to high.
    let mut rst = pins.gpio22.into_push_pull_output();
    rst.set_high().unwrap();

    // Configure the backlight pin, set to high
    let mut bl = pins.gpio20.into_push_pull_output();
    bl.set_high().unwrap();

    rprintln!("Just before initializing display");

    // Initialize the display
    let mut display = Builder::new(models::ST7789, di)
        .reset_pin(rst)
        .display_size(WIDTH as u16, HEIGHT as u16)
        .invert_colors(options::ColorInversion::Inverted)
        .orientation(options::Orientation::default())
        .color_order(options::ColorOrder::Bgr)
        .init(&mut timer)
        .unwrap();

    rprintln!("Display initialized");

    led_pin.set_high().unwrap();
    // Clear the display initially
    display.clear(Rgb565::GREEN).unwrap();

    // Draw a rectangle on screen
    let style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::GREEN)
        .build();

    embedded_graphics::primitives::Rectangle::new(Point::zero(), display.bounding_box().size)
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

    loop {
        // Do nothing
    }
}
