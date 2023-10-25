#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::_export::StaticCell;
use embassy_time::{Duration, Timer};
// use esp_wifi::{initialize, EspWifiInitFor};
pub mod prelude;

#[allow(unused_imports)]
use prelude::*;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::max(system.clock_control).freeze();

    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let led_pin = io.pins.gpio4.into_push_pull_output();
    let led = io.pins.gpio12.into_push_pull_output();

    let ledc = LEDC::new(peripherals.LEDC, &clocks, &mut system.peripheral_clock_control);
    
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    
    embassy::init(&clocks, timer_group0.timer0);

    let executor = EXECUTOR.init(Executor::new());

    logger::init_logger_from_env();
    log::info!("Logger is setup");

    executor.run(|spawner| {
        spawner.spawn(blink(led_pin)).ok();
        spawner.spawn(breathe(led,ledc)).ok();
    })
}

#[embassy_macros::task]
async fn blink(mut led_pin: GpioPin<Output<PushPull>, 4>) {
    loop {
        log::info!("Toggling LED on ...");
        led_pin.set_high().unwrap();
        Timer::after(Duration::from_millis(1500)).await;

        log::info!("Toggling LED off ...");
        led_pin.set_low().unwrap();
        Timer::after(Duration::from_millis(1500)).await;
    }
}

#[embassy_macros::task]
async fn breathe(led: GpioPin<Output<PushPull>, 12>, ledc: LEDC<'_>) {    
    let mut hstimer0 = ledc.get_timer::<HighSpeed>(timer::Number::Timer0);

    hstimer0
    .configure(timer::config::Config {
        duty: timer::config::Duty::Duty5Bit,
        clock_source: timer::HSClockSource::APBClk,
        frequency: 24u32.kHz(),
    })
    .unwrap();

    let mut channel0 = ledc.get_channel(channel::Number::Channel0, led);
    channel0
        .configure(channel::config::Config {
            timer: &hstimer0,
            duty_pct: 10,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();

    channel0.start_duty_fade(0, 100, 2000).expect_err(
        "Fading from 0% to 100%, at 24kHz and 5-bit resolution, over 2 seconds, should fail",
    );
    
    loop {
        // Set up a breathing LED: fade from off to on over a second, then
        // from on back off over the next second.  Then loop.
        channel0.start_duty_fade(0, 100, 1000).unwrap();
        while channel0.is_duty_fade_running() {}
        
        channel0.start_duty_fade(100, 0, 1000).unwrap();
        while channel0.is_duty_fade_running() {}
    }
}