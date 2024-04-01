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

use core::fmt;

pub use embedded_hal::pwm::SetDutyCycle as PwmPin;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

pub static SERVO_CTRL_SIGNAL: Signal<CriticalSectionRawMutex, ServoCommand> = Signal::new();

//ServoCommand
#[derive(fmt::Debug)]
pub enum ServoCommand {
    Pan(i32),
    Tilt(i32),
    Rest(bool),
    PanTilt(i32, i32),
}

pub struct PanTiltServos<P: PwmPin, T: PwmPin> {
    pan: P,
    tilt: T,
}

impl<P: PwmPin, T: PwmPin> PanTiltServos<P, T> {

    /// Create a new `PanTiltServos` instance from the supplied pins
    pub fn new(pan: P, tilt: T) -> Self {
        Self { pan, tilt }
    }
}

pub trait PanTiltServoCtrl {

    type Error: fmt::Debug;

    /// Move the servo pair to the specified coordinates
    fn move_to(&mut self, x: u16, y: u16) -> Result<(), Self::Error>;

    /// Process websocket commands
    fn process_servo_command(&mut self, command: ServoCommand) -> Result<(), Self::Error>;
    
}

impl<P: PwmPin<Error = E>, T: PwmPin<Error = E>, E: fmt::Debug> PanTiltServoCtrl for PanTiltServos<P, T> {

    type Error = E;
    
    fn move_to(&mut self, x: u16, y: u16) -> Result<(), Self::Error> {
        // TODO(mguerrier): double check that this behaves as expected in the real world
        match (self.pan.set_duty_cycle_fraction(x, 180), self.tilt.set_duty_cycle_fraction(y, 180)) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), Ok(())) | (Ok(()), Err(error))=> Err(error),
            (Err(pan_error), Err(tilt_error)) => {
                // TODO(the-wondersmith): propagate both errors as a chain
                todo!()
            }
        }
    }

    
    fn process_servo_command(&mut self, command: ServoCommand) -> Result<(), Self::Error> {
        match command {
            ServoCommand::Rest(_) => todo!(),
            ServoCommand::Pan(value) => todo!(),
            ServoCommand::Tilt(value) => todo!(),
            ServoCommand::PanTilt(x, y) => self.move_to(x as u16, y as u16)
        }
    }
}