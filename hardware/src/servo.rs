//! ## Servo Control Module
//!
//! Adds servo control functionality to any type that implements
//! [OutputPin](embedded_hal::digital::OutputPin), including precise
//! positioning of servos (pan and tilt functionality), automatic
//! angle conversion, and basic command processing.
//!
//! ### Example Usage
//! ```rust
//! # use tkr_hardware::{Servo, ServoCommand};
//! # use embassy_time::{Timer, Duration};
//!
//! async fn servo_control_task(&mut pin: impl Servo) {
//!     loop {
//!         pin.process(ServoCommand::PanTilt(10, 10)).await.map_err(|error| todo!())?;
//!
//!         // Adjust polling interval as needed
//!         Timer::after(Duration::from_millis(10)).await;
//!     }
//! }
//! ```

use core::fmt;

pub use embedded_hal::pwm::SetDutyCycle as PwmPin;

/// Servo Command
///
/// TODO(token-thinker): documentation
#[derive(Copy, Clone, fmt::Debug, serde::Serialize, serde::Deserialize)]
pub enum ServoCommand {
    Pan(i32),
    Tilt(i32),
    Rest(bool),
    PanTilt(i32, i32),
}

/// Servo Pair
///
/// TODO(mguerrier): documentation
pub struct ServoPair<Pan: PwmPin, Tilt: PwmPin> {
    pan: Pan,
    tilt: Tilt,
}

impl<Pan: PwmPin, Tilt: PwmPin> ServoPair<Pan, Tilt> {
    /// Create a new `ServoPair` instance from the supplied pins
    pub fn new(pan: Pan, tilt: Tilt) -> Self {
        Self { pan, tilt }
    }
}

pub trait Servo {
    type Error: fmt::Debug;

    /// Move the servo pair to the specified coordinates
    fn move_to(&mut self, x: u16, y: u16) -> Result<(), Self::Error>;

    /// Process websocket commands
    fn process(&mut self, command: ServoCommand) -> Result<(), Self::Error>;
}

impl<PwmError: fmt::Debug, Pan: PwmPin<Error = PwmError>, Tilt: PwmPin<Error = PwmError>> Servo
    for ServoPair<Pan, Tilt>
{
    type Error = PwmError;

    fn move_to(&mut self, x: u16, y: u16) -> Result<(), Self::Error> {
        // TODO(mguerrier): double check that this behaves as expected in the real world
        match (
            self.pan.set_duty_cycle_fraction(x, 180),
            self.tilt.set_duty_cycle_fraction(y, 180),
        ) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
            (Err(_pan_error), Err(_tilt_error)) => {
                // TODO(the-wondersmith): propagate both errors as a chain
                todo!()
            }
        }
    }

    fn process(&mut self, command: ServoCommand) -> Result<(), Self::Error> {
        match command {
            ServoCommand::Rest(_) => todo!(),
            ServoCommand::Pan(_value) => todo!(),
            ServoCommand::Tilt(_value) => todo!(),
            ServoCommand::PanTilt(x, y) => self.move_to(x as u16, y as u16),
        }
    }
}
