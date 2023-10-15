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
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );

    let clock_cfg = PeripheralClockConfig::with_frequency(&clocks, 40u32.MHz()).unwrap();
    let mut mcpwm = MCPWM::new(peripherals.MCPWM0, clock_cfg, &mut system.peripheral_clock_control);
    
    embassy::init(&clocks, timer_group0.timer0);

    let executor = EXECUTOR.init(Executor::new());

    logger::init_logger_from_env();
    log::info!("Logger is setup");

    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let led_pin = io.pins.gpio4.into_push_pull_output();
    let servo_pin = io.pins.gpio12;

    // connect operator0 to timer0
    mcpwm.operator1.set_timer(&mcpwm.timer1);

    // connect operator0 to gpio12
    let mut pwm_pin = mcpwm.operator1.with_pin_a(servo_pin, PwmPinConfig::UP_ACTIVE_HIGH);

    // start timer with timestamp values in the range of 0..=99 and a frequency of 20 kHz
    let timer_clock_cfg = clock_cfg
        .timer_clock_with_frequency(99, PwmWorkingMode::Increase, 20u32.kHz())
        .unwrap();
    mcpwm.timer1.start(timer_clock_cfg);

    pwm_pin.set_timestamp(50);

    executor.run(|spawner| {
        spawner.spawn(blink(led_pin)).ok();
    })
}

#[embassy_macros::task]
async fn blink(mut len_pin: gpio::GpioPin<gpio::Output<gpio::PushPull>, 4>) {
    loop {
        log::info!("Toggling LED on ...");
        len_pin.set_high().unwrap();
        Timer::after(Duration::from_millis(1500)).await;

        log::info!("Toggling LED off ...");
        len_pin.set_low().unwrap();
        Timer::after(Duration::from_millis(1500)).await;
    }
}
