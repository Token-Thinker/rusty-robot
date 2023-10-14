#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[allow(unused_imports, clippy::single_component_path_imports)]
use esp_backtrace;

use hal::{
    clock::ClockControl,
    embassy,
    embassy::executor::Executor,
    gpio::*,
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup, // Rng,
};

use embassy_executor::_export::StaticCell;
use embassy_time::{Duration, Timer};
// use esp_wifi::{initialize, EspWifiInitFor};

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );

    embassy::init(&clocks, timer_group0.timer0);

    let executor = EXECUTOR.init(Executor::new());

    esp_println::logger::init_logger_from_env();

    log::info!("Logger is setup");

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let led_pin = io.pins.gpio4.into_push_pull_output();

    executor.run(|spawner| {
        spawner.spawn(blink(led_pin)).ok();
    })
}

#[embassy_macros::task]
async fn blink(mut len_pin: GpioPin<Output<PushPull>, 4>) {
    loop {
        log::info!("Toggling LED on ...");
        len_pin.set_high().unwrap();
        Timer::after(Duration::from_millis(1500)).await;

        log::info!("Toggling LED off ...");
        len_pin.set_low().unwrap();
        Timer::after(Duration::from_millis(1500)).await;
    }
}
