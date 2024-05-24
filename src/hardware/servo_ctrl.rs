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
//! loop {
//!    servos.process_servo_command().await.unwrap();
//!    Timer::after(Duration::from_millis(10)).await;
//!  }
//!
//! ```

use core::fmt;

pub use embedded_hal::pwm::SetDutyCycle as PwmPin;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

pub static SERVO_CTRL_SIGNAL: Signal<CriticalSectionRawMutex, ServoCommand> = Signal::new();

//ServoCommand
#[derive(fmt::Debug)]
pub enum ServoCommand {
    Pan(u8),
    Tilt(u8),
    Rest(bool),
    PanTilt(u8, u8),
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
    fn move_to(&mut self, duty_x: u16, duty_y: u16) -> Result<(), Self::Error>;

    /// Process websocket commands
    async fn process_servo_command(&mut self) -> Result<(), Self::Error>;

    ///Helper function to convert angle (0 to 180) to pwm signal based on 14bit
    fn pwm_value(angle: u8) -> u16 { 409 + ((2048 - 409) / 180 * u16::from(angle)) }
}

impl<P: PwmPin<Error = E>, T: PwmPin<Error = E>, E: fmt::Debug> PanTiltServoCtrl for PanTiltServos<P, T> {

    type Error = E;
    
    fn move_to(&mut self, duty_x: u16, duty_y: u16) -> Result<(), Self::Error> {
        match (self.pan.set_duty_cycle(duty_x), self.tilt.set_duty_cycle(duty_y)) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), Ok(())) | (Ok(()), Err(error))=> Err(error),
            (Err(pan_error), Err(tilt_error)) => {
                // TODO(the-wondersmith): propagate both errors as a chain
                todo!()
            }
        }
    }

    async fn process_servo_command(&mut self) -> Result<(), Self::Error> {
        match SERVO_CTRL_SIGNAL.wait().await {
            ServoCommand::Rest(_) => todo!(),
            ServoCommand::Pan(value) => todo!(),
            ServoCommand::Tilt(value) => todo!(),
            ServoCommand::PanTilt(x, y) => self.move_to(Self::pwm_value(x), Self::pwm_value(y))
        }
    }
}