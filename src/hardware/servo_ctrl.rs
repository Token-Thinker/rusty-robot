//! ## Servo Control Module
//!
//! Facilitates precise positioning of servos through angle conversion and command processing, 
//! supporting pan and tilt functionalities.
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

use core::fmt::{self, Error};

use embedded_hal_02::PwmPin;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

pub static SERVO_CTRL_SIGNAL: Signal<CriticalSectionRawMutex, ServoCommand> = Signal::new();

//ServoCommand
#[derive(fmt::Debug)]
pub enum ServoCommand {
    Pan(i32),
    Tilt(i32),
    Rest(bool),
}

struct Servo<P: PwmPin> {
    pwm_pin: P,
    min_pulse: P::Duty,
    max_pulse: P::Duty,
}

trait ServoControl {
    type Error;

    /// Convert the input angle from control to servo's min and max pulse widths
    fn angle_to_duty(angle: u8, min_pulse: u16, max_pulse: u16) -> u16 {
        let duty_range = max_pulse.saturating_sub(min_pulse);
        min_pulse.saturating_add(duty_range.saturating_mul(angle as u16).saturating_div(180))
    }

    /// Sets position of the servo based on the angle and duty
    fn set_position(&mut self, position: u8) -> Result<(), Error>;
}

// Implementation for a specific servo using PWM
impl<P: PwmPin<Duty = u16>> ServoControl for Servo<P>{

    type Error = Error;
    
    fn set_position(&mut self, position: u8) -> Result<(), Error> {
        self.pwm_pin.set_duty(Self::angle_to_duty(position, self.min_pulse, self.max_pulse));
        Ok(())
    }
}

pub struct ServoSystem<P: PwmPin<Duty = u16>> {
    pan_servo: Servo<P>,
    tilt_servo: Servo<P>,
}

// Implementation for Pan & Tilt Servo System
impl<P: PwmPin<Duty = u16> + 'static> ServoSystem<P> {

    /// Map PwmPins to Pan & Tilt for SG90 microservos
    pub fn new_servo_system(pan_pin: P, tilt_pin: P) -> Self {
        let min_pulse = 500; // 0.5ms pulse width
        let max_pulse = 2500; // 2.5ms pulse width

        ServoSystem {
            pan_servo: Servo { pwm_pin: pan_pin, min_pulse, max_pulse },
            tilt_servo: Servo { pwm_pin: tilt_pin, min_pulse, max_pulse },
        }
    }

    /// Function to map joystick values to servo angles
    fn map_value(value: f32, min_in: f32, max_in: f32, min_out: u8, max_out: u8) -> u8 {
        (((value - min_in) * (max_out - min_out) as f32) / (max_in - min_in) + min_out as f32) as u8
    }

    /// Process websocket commands
    pub fn process_command(&mut self, command: ServoCommand) -> Result<(), Error> {
        match command {
            ServoCommand::Pan(value) => {
                self.pan_servo.set_position(
                    Self::map_value(
                        value as f32, 
                        -100.0, 
                        100.0, 
                        0, 
                        180)
                    )
            },
            ServoCommand::Tilt(value) => {
                self.tilt_servo.set_position(Self::map_value(value as f32, -125.0, 125.0, 0, 180))
            }
            ServoCommand::Rest(_) => todo!(),
        }
    }
}