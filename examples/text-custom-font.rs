//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use fugit::RateExtU32;
// use cortex_m::singleton;
use hal::{
    clocks::{ClocksManager, InitError},
    // dma::{double_buffer, single_buffer, DMAExt},
    gpio::{FunctionPio0, Pin},
    pac,
    pac::vreg_and_chip_reset::vreg::VSEL_A,
    pio::{Buffers, PIOExt, ShiftDirection},
    pll::{common_configs::PLL_USB_48MHZ, setup_pll_blocking},
    sio::Sio,
    vreg::set_voltage,
    // watchdog::Watchdog,
    xosc::setup_xosc_blocking,
    Clock,
};
use panic_halt as _;
use rp2040_hal as hal;

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000; // Typically found in BSP crates
use rp_pico as bsp;

use embedded_graphics::{
    image::ImageRaw,
    mono_font::{mapping::StrGlyphMapping, DecorationDimensions, MonoFont, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
const SEVENT_SEGMENT_FONT: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("../assets/seven-segment-font.raw"), 224),
    glyph_mapping: &StrGlyphMapping::new("0123456789", 0),
    character_size: Size::new(22, 40),
    character_spacing: 4,
    baseline: 7,
    underline: DecorationDimensions::default_underline(40),
    strikethrough: DecorationDimensions::default_strikethrough(40),
};
use lib::{overclock, Pio16BitBus, ILI9488};
use overclock::overclock_configs::PLL_SYS_240MHZ;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    // let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    set_voltage(&mut pac.VREG_AND_CHIP_RESET, VSEL_A::VOLTAGE1_10);

    let xosc = setup_xosc_blocking(pac.XOSC, XOSC_CRYSTAL_FREQ.Hz())
        .map_err(InitError::XoscErr)
        .ok()
        .unwrap();
    let mut clocks = ClocksManager::new(pac.CLOCKS);

    let pll_sys = setup_pll_blocking(
        pac.PLL_SYS,
        xosc.operating_frequency().into(),
        PLL_SYS_240MHZ,
        &mut clocks,
        &mut pac.RESETS,
    )
    .map_err(InitError::PllError)
    .unwrap();
    let pll_usb = setup_pll_blocking(
        pac.PLL_USB,
        xosc.operating_frequency().into(),
        PLL_USB_48MHZ,
        &mut clocks,
        &mut pac.RESETS,
    )
    .map_err(InitError::PllError)
    .unwrap();

    clocks
        .init_default(&xosc, &pll_sys, &pll_usb)
        .map_err(InitError::ClockError)
        .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let program = pio_proc::pio_asm!(
        ".side_set 1"
        ".wrap_target",
        "   out pins, 16    side 0",
        "   nop             side 1",
        ".wrap"
    );

    let wr: Pin<_, FunctionPio0, _> = pins.gpio19.into_function();
    let wr_pin_id = wr.id().num;

    let dc = pins.gpio20.into_push_pull_output();
    let rst = pins.gpio22.into_push_pull_output();
    let bl = pins.gpio28.into_push_pull_output();

    let lcd_d0: Pin<_, FunctionPio0, _> = pins.gpio0.into_function();
    let lcd_d1: Pin<_, FunctionPio0, _> = pins.gpio1.into_function();
    let lcd_d2: Pin<_, FunctionPio0, _> = pins.gpio2.into_function();
    let lcd_d3: Pin<_, FunctionPio0, _> = pins.gpio3.into_function();
    let lcd_d4: Pin<_, FunctionPio0, _> = pins.gpio4.into_function();
    let lcd_d5: Pin<_, FunctionPio0, _> = pins.gpio5.into_function();
    let lcd_d6: Pin<_, FunctionPio0, _> = pins.gpio6.into_function();
    let lcd_d7: Pin<_, FunctionPio0, _> = pins.gpio7.into_function();
    let lcd_d8: Pin<_, FunctionPio0, _> = pins.gpio8.into_function();
    let lcd_d9: Pin<_, FunctionPio0, _> = pins.gpio9.into_function();
    let lcd_d10: Pin<_, FunctionPio0, _> = pins.gpio10.into_function();
    let lcd_d11: Pin<_, FunctionPio0, _> = pins.gpio11.into_function();
    let lcd_d12: Pin<_, FunctionPio0, _> = pins.gpio12.into_function();
    let lcd_d13: Pin<_, FunctionPio0, _> = pins.gpio13.into_function();
    let lcd_d14: Pin<_, FunctionPio0, _> = pins.gpio14.into_function();
    let lcd_d15: Pin<_, FunctionPio0, _> = pins.gpio15.into_function();

    let lcd_d0_pin_id = lcd_d0.id().num;

    let pindirs = [
        (wr_pin_id, hal::pio::PinDir::Output),
        (lcd_d0.id().num, hal::pio::PinDir::Output),
        (lcd_d1.id().num, hal::pio::PinDir::Output),
        (lcd_d2.id().num, hal::pio::PinDir::Output),
        (lcd_d3.id().num, hal::pio::PinDir::Output),
        (lcd_d4.id().num, hal::pio::PinDir::Output),
        (lcd_d5.id().num, hal::pio::PinDir::Output),
        (lcd_d6.id().num, hal::pio::PinDir::Output),
        (lcd_d7.id().num, hal::pio::PinDir::Output),
        (lcd_d8.id().num, hal::pio::PinDir::Output),
        (lcd_d9.id().num, hal::pio::PinDir::Output),
        (lcd_d10.id().num, hal::pio::PinDir::Output),
        (lcd_d11.id().num, hal::pio::PinDir::Output),
        (lcd_d12.id().num, hal::pio::PinDir::Output),
        (lcd_d13.id().num, hal::pio::PinDir::Output),
        (lcd_d14.id().num, hal::pio::PinDir::Output),
        (lcd_d15.id().num, hal::pio::PinDir::Output),
    ];

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let installed = pio.install(&program.program).unwrap();
    let (int, frac) = (1, 0); // as fast as possible
    let (mut sm, _, tx) = rp2040_hal::pio::PIOBuilder::from_installed_program(installed)
        .side_set_pin_base(wr_pin_id)
        .out_pins(lcd_d0_pin_id, 16)
        .buffers(Buffers::OnlyTx)
        .clock_divisor_fixed_point(int, frac)
        .out_shift_direction(ShiftDirection::Right)
        .autopull(true)
        .pull_threshold(16)
        .build(sm0);
    sm.set_pindirs(pindirs);
    sm.start();

    info!("PIO block setuped");

    let di = Pio16BitBus::new(tx, dc);
    let mut display = ILI9488::new(di, Some(rst), Some(bl), 480, 320);
    display.init(&mut delay).unwrap();

    let light_blue = Rgb888::new(0x00, 0xd2, 0xff);
    let dark_blue = Rgb888::new(0x00, 0x14, 0x28);
    display.color_converted().clear(dark_blue).unwrap();
    let character_style = MonoTextStyle::new(&SEVENT_SEGMENT_FONT, light_blue);
    let text_style = TextStyleBuilder::new()
        .baseline(Baseline::Bottom)
        .alignment(Alignment::Center)
        .build();

    Text::with_text_style(
        "123\n456",
        display.bounding_box().center(),
        character_style,
        text_style,
    )
    .draw(&mut display.color_converted())
    .unwrap();
    loop {
        cortex_m::asm::wfi();
    }
}

// End of file
