//! ## Servo Control Module
//!
//! Facilitates precise positioning of servos through angle conversion and command processing, 
//! supporting pan and tilt functionalities.
//!
//! **Key Functions**
//!
//! * `new()`: Initializes the servo system with specified PWM pins for pan and tilt servos.
//! * `set_position()`: Directly sets a servo's position to a specified angle.
//! * `process_command()`: Interprets and executes pan, tilt, and rest commands.
//!
//! **Error Handling**
//! * **ServoError:** Handles issues like invalid pulse widths. Extend as needed for more detailed error management.
//!
//! **Usage**
//! ```rust
//! use servo_control::*
//!
//! let mut servo_system = ServoSystem::new(pan_pin, tilt_pin);
//!
//! // To pan to 45 degrees
//! servo_system.process_command(ServoCommand::Pan(45)).expect("Pan command failed");
//!
//! // To tilt to 90 degrees
//! servo_system.process_command(ServoCommand::Tilt(90)).expect("Tilt command failed");
//!
//! // Move servos to a rest position or disable them
//! servo_system.process_command(ServoCommand::Rest(true)).expect("Rest command failed");
//! ```

use core::fmt;

use embedded_hal_02::PwmPin;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

pub static SERVO_CTRL_SIGNAL: Signal<CriticalSectionRawMutex, ServoCommand> = Signal::new();

pub enum ServoCommand {
    Pan(i32),
    Tilt(i32),
    Rest(bool),
}

/// Error handling
#[derive(fmt::Debug)]
pub enum ServoError {
    InvalidPulseWidth,
}

trait ServoControl {
    fn angle_to_duty(angle: u8, min_pulse: u16, max_pulse: u16) -> u16;
    fn set_position(&mut self, position: u8) -> Result<(), ServoError>;
}

// Implementation for a specific servo using PWM
struct Servo<P: PwmPin> {
    pwm_pin: P,
    min_pulse: P::Duty,
    max_pulse: P::Duty,
}

impl<P: PwmPin<Duty = u16>> ServoControl for Servo<P>{

    fn set_position(&mut self, position: u8) -> Result<(), ServoError> {
        let duty = Self::angle_to_duty(position, self.min_pulse, self.max_pulse);
        self.pwm_pin.set_duty(duty);
        Ok(())
    }

    // Adjusted function to use the servo's min and max pulse widths
    fn angle_to_duty(angle: u8, min_pulse: u16, max_pulse: u16) -> u16 {
        let duty_range = max_pulse as u32 - min_pulse as u32;
        let duty = min_pulse as u32 + (duty_range * angle as u32 / 180);
        duty as u16
    }
}

pub struct ServoSystem<P: PwmPin<Duty = u16>> {
    pan_servo: Servo<P>,
    tilt_servo: Servo<P>,
}

impl<P: PwmPin<Duty = u16> + 'static> ServoSystem<P> {
    pub fn new(pan_pin: P, tilt_pin: P) -> Self {
        let min_pulse = 500; // 0.5ms pulse width
        let max_pulse = 2500; // 2.5ms pulse width

        ServoSystem {
            pan_servo: Servo { pwm_pin: pan_pin, min_pulse, max_pulse },
            tilt_servo: Servo { pwm_pin: tilt_pin, min_pulse, max_pulse },
        }
    }

    // Function to map joystick values to servo angles
    fn map_value(value: f32, min_in: f32, max_in: f32, min_out: u8, max_out: u8) -> u8 {
        (((value - min_in) * (max_out - min_out) as f32) / (max_in - min_in) + min_out as f32) as u8
    }

    pub fn process_command(&mut self, command: ServoCommand) -> Result<(), ServoError> {
        match command {
            ServoCommand::Pan(value) => {
                let pan_position = Self::map_value(value as f32, -100.0, 100.0, 0, 180);
                self.pan_servo.set_position(pan_position)
            },
            ServoCommand::Tilt(value) => {
                let tilt_position = Self::map_value(value as f32, -125.0, 125.0, 0, 180);
                self.tilt_servo.set_position(tilt_position)
            }
            ServoCommand::Rest(_) => todo!(),
        }
    }
}