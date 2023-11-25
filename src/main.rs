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

use embassy_time::Timer;
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
    mut led: GpioPin<Output<PushPull>, 4>,
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
    servo_pan: GpioPin<Output<PushPull>, 16>,
    servo_tilt: GpioPin<Output<PushPull>, 13>,
    ledc: &'static LEDC<'_>,
){
    println!("Initializing control_servo task");

    let mut hstimer1 = ledc.get_timer::<HighSpeed>(timer::Number::Timer1);
    println!("Timer configured");

    hstimer1
    .configure(timer::config::Config {
        duty: timer::config::Duty::Duty14Bit,
        clock_source: timer::HSClockSource::APBClk,
        frequency: 50u32.Hz(),
    })
    .unwrap();
    println!("Timer initialization complete");

    // Function to convert angle to duty cycle
    fn angle_to_duty(angle: u8) -> u32 {
        let min_pulse_ms = 0.5; // 1ms pulse width for 0 degrees
        let max_pulse_ms = 2.4; // 2ms pulse width for 180 degrees
    
        let min_duty = (min_pulse_ms / 20.0) * 16383.0;
        let max_duty = (max_pulse_ms / 20.0) * 16383.0;
    
        let duty_range = max_duty - min_duty;
        let duty_14bit = min_duty + (duty_range * (angle as f32 / 180.0));
    
        duty_14bit as u32
    }

    let mut channel1 = ledc.get_channel(channel::Number::Channel1, servo_pan);
    let mut channel2 = ledc.get_channel(channel::Number::Channel1, servo_tilt);
    println!("Channel configured");

    channel1
        .configure(channel::config::Config {
            timer: &hstimer1,
            duty_pct: 10,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();

    channel2
    .configure(channel::config::Config {
        timer: &hstimer1,
        duty_pct: 10,
        pin_config: channel::config::PinConfig::PushPull,
    })
    .unwrap();
    println!("Channel initialization complete, starting sweep");

    // Sweeping loop
    loop {
        // Sweep from 0 to 180 degrees
        for angle in 0..=180u8 {
            let duty = angle_to_duty(angle);
            channel1.set_duty_hw(duty);
            channel2.set_duty_hw(duty);
            println!("Sweeping up - Angle: {}, Duty: {}", angle, duty);
            Timer::after(Duration::from_millis(10)).await;
        }

        // Sweep back from 180 to 0 degrees
        for angle in (0..=180u8).rev() {
            let duty = angle_to_duty(angle);
            channel1.set_duty_hw(duty);
            channel2.set_duty_hw(duty);
            println!("Sweeping down - Angle: {}, Duty: {}", angle, duty);
            Timer::after(Duration::from_millis(10)).await;
        }
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

    // Sends periodic messages to control_led, enabling or disabling it.
    println!(
        "Starting enable_disable_led() on core {}",
        get_core() as usize
    );
    let mut ticker = Ticker::every(Duration::from_secs(1));

    _spawner.spawn(control_servo(servo_pan,servo_tilt, ledc)).ok();

    loop {
        esp_println::println!("Sending LED on");
        led_ctrl_signal.signal(true);
        ticker.next().await;

        esp_println::println!("Sending LED off");
        led_ctrl_signal.signal(false);
        ticker.next().await;
    }
}

