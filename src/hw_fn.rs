//! ## TKR's `Hardware Functions` Module
//!
//! This "module" contains embassy async tasks for controlling
//! hardware functionality of the TKR robot. 
//! 
//! Hardware GPIO Structure
//!     GPIO 2 - Flywheel Motors
//!     GPIO 12 - Launcher Servo
//!     GPIO 13 - Tilt Servo
//!     GPIO 16 - Pan Servo
//! 
//! Functions
//!     [ ] Turn GPIO2 High/Low via Signal
//!     [ ] Push Liner Servo Motor back and for for reload
//!     [ ] Set Postion of Tilt Servo (Sync)
//!     [ ] Set Postion of Pan Servo (Sync)
//!  

#[allow(unused_imports)]
use crate::prelude::*;
extern crate alloc;


//Control Flywheel Motors Task  ####################################################################################################

#[embassy_executor::task]
pub async fn enable_disable_led(control: &'static Signal<CriticalSectionRawMutex, bool>) {
    println!(
        "Starting enable_disable_led() on core {}",
        get_core() as usize
    );
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        esp_println::println!("Sending LED on");
        control.signal(true);
        ticker.next().await;

        esp_println::println!("Sending LED off");
        control.signal(false);
        ticker.next().await;
    }
}

#[embassy_executor::task]
pub async fn control_led(
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

//#######################################################################################################################



// Control Servo Task ##################################################################################################
#[embassy_executor::task]
pub async fn control_servo(
    servo_tilt: GpioPin<Output<PushPull>, 13>,
    servo_pan: GpioPin<Output<PushPull>, 16>,    
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

    // Function to map joystick values to servo angles
    fn map_value(value: f32, min_in: f32, max_in: f32, min_out: u8, max_out: u8) -> u8 {
        (((value - min_in) * (max_out - min_out) as f32) / (max_in - min_in) + min_out as f32) as u8
    }

    let mut channel1: channel::Channel<'_, HighSpeed, GpioPin<Output<PushPull>, 16>> = ledc.get_channel(channel::Number::Channel1, servo_pan);
    let mut channel2 = ledc.get_channel(channel::Number::Channel2, servo_tilt);
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

    // Function to handle command
    fn handle_command(command: &str, channel1: &mut channel::Channel<'_, HighSpeed, GpioPin<Output<PushPull>, 16>>, channel2: &mut channel::Channel<'_, HighSpeed, GpioPin<Output<PushPull>, 13>>) {
        let parts: alloc::vec::Vec< &str>  = command.split(':').collect();
        if parts.len() == 2 {
            if let (Ok(x_axis_value), Ok(y_axis_value)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                let servo1_position = map_value(x_axis_value, -100.0, 100.0, 0, 180);
                let servo2_position = map_value(y_axis_value, -125.0, 125.0, 0, 180);

                let duty1 = angle_to_duty(servo1_position);
                let duty2 = angle_to_duty(servo2_position);

                channel1.set_duty_hw(duty1);
                channel2.set_duty_hw(duty2);

                println!("Servo 1 Position: {}, Servo 2 Position: {}", servo1_position, servo2_position);
            }
        }
    }

    // Here you call the function with a sample command "50:-30"
    handle_command("50:-30", &mut channel1, &mut channel2);

}

//#######################################################################################################################