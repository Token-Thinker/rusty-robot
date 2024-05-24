#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]
#![feature(type_alias_impl_trait)]

pub mod hardware;
pub mod network;
pub mod prelude;

use hardware::{servo_ctrl::*, motor_ctrl::*};
use network::ntwk;
#[allow(unused_imports)]
use prelude::*;

#[async_task]
async fn motor_control_task(mut pin: impl Motor + 'static) {
    loop {
        pin.process_command().await.unwrap();
        Timer::after(Duration::from_millis(10)).await;
    }

}

#[async_task]
async fn servo_control_task(mut servos: impl PanTiltServoCtrl + 'static) {

    loop {
        servos.process_servo_command().await.unwrap();
        Timer::after(Duration::from_millis(10)).await;
    }
}

#[cfg(all(target_os = "none", target_arch = "xtensa", target_vendor = "unknown"))]
#[main]
async fn main(_spawner: Spawner) {
    init_heap();

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // initialize emabassy
    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    //Network Services Configurations
    let timer = TimerGroup::new(peripherals.TIMG1, &clocks, None).timer0;
    let config = Config::dhcpv4(Default::default());
    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    ).unwrap();

    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

    let seed = 1234; // very random, very secure seed

    // Init network stack
    let stack = &*make_static!(Stack::new(
        wifi_interface,
        config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    //initialize pins
    let pin = io.pins.gpio4.into_push_pull_output();
    let pan_pin = io.pins.gpio12.into_push_pull_output();
    let tilt_pin = io.pins.gpio14.into_push_pull_output();

    //initialize ledc
    let ledc = make_static!(LEDC::new(peripherals.LEDC, make_static!(clocks)));
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    //initialize timer
    let lstimer1 = make_static!(ledc.get_timer::<LowSpeed>(timer::Number::Timer1));

    lstimer1
    .configure(timer::config::Config {
        duty: timer::config::Duty::Duty14Bit,
        clock_source: timer::LSClockSource::APBClk,
        frequency: 50u32.Hz(),
    })
    .unwrap();

    //configure channel
    let mut channel1 = ledc.get_channel(channel::Number::Channel1, pan_pin);
    let mut channel2 = ledc.get_channel(channel::Number::Channel2, tilt_pin);

    channel1
        .configure(channel::config::Config {
            timer: lstimer1,
            duty_pct: 10,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();

    channel2
        .configure(channel::config::Config {
            timer: lstimer1,
            duty_pct: 10,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();

    let servos = PanTiltServos::new(channel1, channel2);

    //network tasks
    _spawner.spawn(ntwk::connection(controller)).ok();
    _spawner.spawn(ntwk::net_task(stack)).ok();

    //hardware task
    _spawner.spawn(motor_control_task(pin)).ok();
    _spawner.spawn(servo_control_task(servos)).ok();

}