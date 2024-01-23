#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod hardware;
pub mod network;
pub mod prelude;

use hardware::hw::*;
use network::server::*;
#[allow(unused_imports)]
use prelude::*;

use core::mem::MaybeUninit;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

#[main]
async fn main(_spawner: Spawner) {
    init_heap();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);

    //Embassy Configurations
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0.timer0);

    //Network Services Configurations
    let timer = TimerGroup::new(peripherals.TIMG1, &clocks).timer0;
    let config = Config::dhcpv4(Default::default());
    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    let wifi = peripherals.WIFI;

    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

    let seed = 1234;

    // Init network stack
    let stack = &*make_static!(Stack::new(
        wifi_interface,
        config,
        make_static!(StackResources::<4>::new()),
        seed
    ));

    let pico_config = make_static!(picoserve::Config::new(picoserve::Timeouts {
        start_read_request: Some(Duration::from_secs(5)),
        read_request: Some(Duration::from_secs(1)),
        write: Some(Duration::from_secs(5)),
    })
    .keep_connection_alive());

    //Flywheel Motor Configurations
    let led = io.pins.gpio4.into_push_pull_output();

    //Servo Motors Configurations
    let ledc = make_static!(LEDC::new(peripherals.LEDC, make_static!(clocks)));
    let servo_tilt = io.pins.gpio13.into_push_pull_output();
    let servo_pan = io.pins.gpio16.into_push_pull_output();

    //Spawner Functions
    _spawner.spawn(connection(controller)).ok();
    _spawner.spawn(net_task(stack)).ok();
    _spawner.spawn(server(stack, pico_config, _spawner)).ok();

    _spawner.spawn(control_motor(led)).ok();
    _spawner.spawn(control_servo(servo_tilt, servo_pan, ledc)).ok();
}
