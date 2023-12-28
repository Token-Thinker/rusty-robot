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
//!     [X] Turn GPIO2 High/Low via Signal
//!     [ ] Push Liner Servo Motor back and for for reload
//!     [X] Set Postion of Tilt Servo (Sync)
//!     [X] Set Postion of Pan Servo (Sync)
//!  

#[allow(unused_imports)]
use crate::prelude::*;
extern crate alloc;
use crate::network::sginal::*;

#[embassy_executor::task]
pub async fn control_motor(mut led: GpioPin<Output<PushPull>, 4>) {
    loop {
        // Wait for a command signal
        let command = MOTOR_CTRL_SIGNAL.wait().await;

        match command {
            MotorCommand::On => {
                led.set_high().unwrap();
            }
            MotorCommand::Off => {
                led.set_low().unwrap();
            }
            MotorCommand::Launch => {
                for _ in 0..100 {
                    led.toggle().unwrap();
                    Timer::after(Duration::from_millis(1)).await;
                    led.toggle().unwrap();
                    Timer::after(Duration::from_millis(1)).await;
                }
            }
        }
    }
}

#[embassy_executor::task]
pub async fn control_servo(
    servo_tilt: GpioPin<Output<PushPull>, 13>,
    servo_pan: GpioPin<Output<PushPull>, 16>,
    ledc: &'static LEDC<'_>,
) {
    println!("Initializing control_servo task");
    let mut current_pan_position: f32 = 90.0;
    let mut current_tilt_position: f32 = 90.0;

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

    let mut pan_channel: channel::Channel<'_, HighSpeed, GpioPin<Output<PushPull>, 16>> =
        ledc.get_channel(channel::Number::Channel1, servo_pan);
    let mut tilt_channel = ledc.get_channel(channel::Number::Channel2, servo_tilt);

    println!("Channel configured");

    pan_channel
        .configure(channel::config::Config {
            timer: &hstimer1,
            duty_pct: 10,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();

    tilt_channel
        .configure(channel::config::Config {
            timer: &hstimer1,
            duty_pct: 10,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();
    println!("Channel initialization complete");

    // Function to handle command
    async fn handle_command(
        command: ServoCommand,
        pan_channel: &mut channel::Channel<'_, HighSpeed, GpioPin<Output<PushPull>, 16>>,
        tilt_channel: &mut channel::Channel<'_, HighSpeed, GpioPin<Output<PushPull>, 13>>,
        current_pan_position: &mut f32,
        current_tilt_position: &mut f32,
    ) {
        if command.at_rest {
            // Do not move servos if the joystick is at rest
            return;
        }

        let target_pan_position = map_value(command.dx as f32, -100.0, 100.0, 0, 180) as f32;
        let target_tilt_position = map_value(command.dy as f32, -125.0, 125.0, 0, 180) as f32;

        // Calculate the steps needed to move smoothly
        const INTERPOLATION_STEPS: u8 = 10;
        let step_pan = (target_pan_position - *current_pan_position) / INTERPOLATION_STEPS as f32;
        let step_tilt =
            (target_tilt_position - *current_tilt_position) / INTERPOLATION_STEPS as f32;

        for _ in 0..INTERPOLATION_STEPS {
            *current_pan_position += step_pan;
            *current_tilt_position += step_tilt;

            let duty1 = angle_to_duty(*current_pan_position as u8);
            let duty2 = angle_to_duty(*current_tilt_position as u8);

            pan_channel.set_duty_hw(duty1);
            tilt_channel.set_duty_hw(duty2);

            // Delay for smooth movement
            Timer::after(Duration::from_millis(20)).await;
        }

        // After movement, update the current position
        *current_pan_position = target_pan_position;
        *current_tilt_position = target_tilt_position;

        println!(
            "Pan Position: {}, Tilt Position: {}",
            target_pan_position, target_tilt_position
        );
    }

    loop {
        let servo_command = SERVO_CTRL_SIGNAL.wait().await;

        // Pass the current position state as mutable references
        handle_command(
            servo_command,
            &mut pan_channel,
            &mut tilt_channel,
            &mut current_pan_position,
            &mut current_tilt_position,
        )
        .await;
    }
}
