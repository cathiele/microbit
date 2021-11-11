//! A complete working example.
//!
//! This requires `cortex-m-rtfm` v0.5.
//!
//! It uses `TIMER1` to drive the display, and `RTC0` to update a simple
//! animated image.
#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_halt as _;

use microbit::{
    display::nonblocking::{Display, GreyscaleImage},
    gpio::DisplayPins,
    hal::{
        clocks::Clocks,
        gpio::{p0, p1, Level},
        rtc::{Rtc, RtcInterrupt},
    },
    pac,
};

use rtic::app;

fn heart_image(inner_brightness: u8) -> GreyscaleImage {
    let b = inner_brightness;
    GreyscaleImage::new(&[
        [0, 7, 0, 7, 0],
        [7, b, 7, b, 7],
        [7, b, b, b, 7],
        [0, 7, b, 7, 0],
        [0, 0, 7, 0, 0],
    ])
}

#[app(device = microbit::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        display: Display<pac::TIMER1>,
        anim_timer: Rtc<pac::RTC0>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        // Starting the low-frequency clock (needed for RTC to work)
        let device = cx.device;
        Clocks::new(device.CLOCK).start_lfclk();

        // RTC at 16Hz (32_768 / (2047 + 1))
        // 16Hz; 62.5ms period
        let mut rtc0 = Rtc::new(device.RTC0, 2047).unwrap();
        rtc0.enable_event(RtcInterrupt::Tick);
        rtc0.enable_interrupt(RtcInterrupt::Tick, None);
        rtc0.enable_counter();

        let p0parts = p0::Parts::new(device.P0);
        let p1parts = p1::Parts::new(device.P1);
        let display_pins = DisplayPins {
            col1: p0parts.p0_28.into_push_pull_output(Level::High),
            col2: p0parts.p0_11.into_push_pull_output(Level::High),
            col3: p0parts.p0_31.into_push_pull_output(Level::High),
            col4: p1parts.p1_05.into_push_pull_output(Level::High),
            col5: p0parts.p0_30.into_push_pull_output(Level::High),
            row1: p0parts.p0_21.into_push_pull_output(Level::Low),
            row2: p0parts.p0_22.into_push_pull_output(Level::Low),
            row3: p0parts.p0_15.into_push_pull_output(Level::Low),
            row4: p0parts.p0_24.into_push_pull_output(Level::Low),
            row5: p0parts.p0_19.into_push_pull_output(Level::Low),
        };

        let display = Display::new(device.TIMER1, display_pins);

        init::LateResources {
            anim_timer: rtc0,
            display,
        }
    }

    #[task(binds = TIMER1, priority = 2, resources = [display])]
    fn timer1(cx: timer1::Context) {
        cx.resources.display.handle_display_event();
    }

    #[task(binds = RTC0, priority = 1, resources = [anim_timer, display])]
    fn rtc0(mut cx: rtc0::Context) {
        static mut STEP: u8 = 0;

        cx.resources.anim_timer.reset_event(RtcInterrupt::Tick);

        let inner_brightness = match *STEP {
            0..=8 => 9 - *STEP,
            9..=12 => 0,
            _ => unreachable!(),
        };

        cx.resources.display.lock(|display| {
            display.show(&heart_image(inner_brightness));
        });

        *STEP += 1;
        if *STEP == 13 {
            *STEP = 0
        };
    }
};
