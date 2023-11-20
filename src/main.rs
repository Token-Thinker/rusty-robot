#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use esp_println::println;
use hal::get_core;

pub mod prelude;

use embassy_executor::Spawner;
use static_cell::make_static;

static mut APP_CORE_STACK: Stack<8192> = Stack::new();


#[allow(unused_imports)]
use prelude::*;

/*#[embassy_macros::task]
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
*/
#[embassy_executor::task]
async fn control_led(
    mut led: GpioPin<Output<PushPull>, 0>,
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

/* #[embassy_macros::task]
async fn breathe(channel0: hal::ledc::channel::Channel<'static, HighSpeed, GpioPin<Output<PushPull>, 4>> ) {        
    loop {
        channel0.start_duty_fade(0, 100, 1000).unwrap();
        while channel0.is_duty_fade_running() {}
        
        channel0.start_duty_fade(100, 0, 1000).unwrap();
        while channel0.is_duty_fade_running() {}
    }
} */


#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0, 
        &clocks,
        &mut system.peripheral_clock_control,
    );
    embassy::init(&clocks, timer_group0.timer0);

    // Set GPIO2 as an output, and set its state high initially.
    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut cpu_control = CpuControl::new(system.cpu_control);

    let led_ctrl_signal = &*make_static!(Signal::new());

    let led = io.pins.gpio0.into_push_pull_output();
    let cpu1_fnctn = move || {
        let executor = make_static!(Executor::new());
        executor.run(|spawner| {
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
    
    loop {
        esp_println::println!("Sending LED on");
        led_ctrl_signal.signal(true);
        ticker.next().await;

        esp_println::println!("Sending LED off");
        led_ctrl_signal.signal(false);
        ticker.next().await;
    }
}

/* #[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();

    let clocks = make_static!(ClockControl::max(system.clock_control).freeze());

    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let led = io.pins.gpio4.into_push_pull_output();

    let ledc = make_static!(LEDC::new(peripherals.LEDC, clocks, &mut system.peripheral_clock_control));
    let hstimer0 = make_static!(ledc.get_timer::<HighSpeed>(timer::Number::Timer0));

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
            timer: hstimer0,
            duty_pct: 10,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();

    channel0.start_duty_fade(0, 100, 2000).expect_err(
        "Fading from 0% to 100%, at 24kHz and 5-bit resolution, over 2 seconds, should fail",
    );

    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    
    embassy::init(&clocks, timer_group0.timer0);

    

    let mut cpu_control = CpuControl::new(system.cpu_control);
    let led_ctrl_signal = &*make_static!(Signal::new());

    let cpu1_fnctn = move || {
        let executor = make_static!(Executor::new());
        executor.run(|spawner| {
            //spawner.spawn(blink(led_pin)).ok();
            spawner.spawn(breathe(channel0)).ok();
        })
    };

    logger::init_logger_from_env();
    log::info!("Logger is setup");

    loop {
        todo!();
    }

} */

