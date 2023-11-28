/* 

Hardware - GPIO structure
GPIO2 - Flywheel Motors (High/Low)
GPI012 - Launcher Servo
GPIO13 - Tilt Servo
GPIO16 - Pan Servo
 
Aux - GPIOs
GPIO4 - LED indicator Feedback
GPIO 2/4/12/13/14/15 - microSD Recording subprocess [requires other gpios to be disabled] 

*/

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod prelude;
pub mod network_access;
pub mod new;


/* static mut APP_CORE_STACK: hal_stack<8192> = hal_stack::new();

static CLOCKS: StaticCell<Clocks> = StaticCell::new(); */


#[allow(unused_imports)]
use prelude::*;
use network_access::new_network_service;
//use new::{control_led, control_servo,enable_disable_led};

#[main]
async fn main(_spawner: Spawner){
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    //let clocks = CLOCKS.init(ClockControl::max(system.clock_control).freeze());
    let clocks = ClockControl::max(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks );
    let timer = TimerGroup::new(peripherals.TIMG1, &clocks).timer0;


    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    ).unwrap();

    let wifi = peripherals.WIFI;
    let (wifi_ap_interface, wifi_sta_interface, wifi_controller) =
        esp_wifi::wifi::new_ap_sta(&init, wifi).unwrap();

    embassy::init(&clocks, timer_group0.timer0);

/*     let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut cpu_control = CpuControl::new(system.cpu_control);
    let led_ctrl_signal = &*make_static!(Signal::new());

    let led = io.pins.gpio4.into_push_pull_output();
    let servo_pan = io.pins.gpio16.into_push_pull_output();
    let servo_tilt = io.pins.gpio13.into_push_pull_output();
    let ledc = make_static!(LEDC::new(peripherals.LEDC, clocks));

    let cpu1_fnctn = move || {
        let executor_cpu1 = make_static!(Executor::new());
        executor_cpu1.run(|spawner| {
            spawner.spawn(control_led(led, led_ctrl_signal)).ok();
        });
    };

    let _guard = cpu_control
        .start_app_core(unsafe { &mut APP_CORE_STACK }, cpu1_fnctn)
        .unwrap();

    _spawner.spawn(enable_disable_led(led_ctrl_signal)).ok();
    _spawner.spawn(control_servo(servo_pan,servo_tilt, ledc)).ok(); */
    _spawner.spawn(new_network_service(_spawner, wifi_ap_interface, wifi_sta_interface, wifi_controller)).ok();
}

