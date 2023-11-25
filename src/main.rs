#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use esp_println::println;
use hal::{get_core, clock::Clocks};

pub mod prelude;

use embassy_executor::Spawner;
use hal::embassy::executor::Executor;
use static_cell::{make_static, StaticCell};

static mut APP_CORE_STACK: Stack<8192> = Stack::new();

static CLOCKS: StaticCell<Clocks> = StaticCell::new();


#[allow(unused_imports)]
use prelude::*;

#[embassy_executor::task]
async fn control_led(
    mut led: GpioPin<Output<PushPull>, 12>,
    control: &'static Signal<CriticalSectionRawMutex, bool>,
) {
    println!("Starting control_led() on core {}", get_core() as usize);
    loop {
        if control.wait().await {
            esp_println::println!("LED on");
            led.set_low().unwrap();
        } else {
            esp_println::println!("LED off");
            led.set_high().unwrap();
        }
    }
}

#[embassy_executor::task]
async fn control_servo(
    servo: GpioPin<Output<PushPull>, 4>,
    ledc: &'static LEDC<'_>,
){
    let mut hstimer0 = ledc.get_timer::<HighSpeed>(timer::Number::Timer0);

    hstimer0
    .configure(timer::config::Config {
        duty: timer::config::Duty::Duty5Bit,
        clock_source: timer::HSClockSource::APBClk,
        frequency: 24u32.kHz(),
    })
    .unwrap();

    let mut channel0 = ledc.get_channel(channel::Number::Channel0, servo);
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
        channel0.start_duty_fade(0, 100, 1000).unwrap();
        while channel0.is_duty_fade_running() {}
        
        channel0.start_duty_fade(100, 0, 1000).unwrap();
        while channel0.is_duty_fade_running() {}
    }

}


#[main]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = CLOCKS.init(ClockControl::max(system.clock_control).freeze());

    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0, 
        &clocks
    );
    embassy::init(&clocks, timer_group0.timer0);

    // Set GPIO2 as an output, and set its state high initially.
    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut cpu_control = CpuControl::new(system.cpu_control);
    let led_ctrl_signal = &*make_static!(Signal::new());

    let led = io.pins.gpio12.into_push_pull_output();
    let servo = io.pins.gpio4.into_push_pull_output();
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

    // Sends periodic messages to control_led, enabling or disabling it.
    println!(
        "Starting enable_disable_led() on core {}",
        get_core() as usize
    );
    let mut ticker = Ticker::every(Duration::from_secs(1));

    _spawner.spawn(control_servo(servo, ledc)).ok();

    loop {
        esp_println::println!("Sending LED on");
        led_ctrl_signal.signal(true);
        ticker.next().await;

        esp_println::println!("Sending LED off");
        led_ctrl_signal.signal(false);
        ticker.next().await;
    }
}

