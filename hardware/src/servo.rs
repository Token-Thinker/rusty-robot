//! ## Servo Control Module
//!
//! Facilitates precise positioning of servos through angle conversion and command processing,
//! supporting pan and tilt functionalities for [PwmPin](embedded_hal::pwm::SetDutyCycle as PwmPin;)
//!
use core::fmt;

pub use embedded_hal::pwm::SetDutyCycle as PwmPin;

/// Servo Command
///
/// Variants:
/// - `Pan(u8)`: Pan the servo to a specific angle.
///     - Ex: `{ "Servo": { "Pan": 30 } }`
/// - `Tilt(u8)`: Tilt the servo to a specific angle.
///     - Ex: `{ "Servo": { "Tilt": 45 } }`
/// - `Rest(bool)`: Set the servo to a resting state (true for rest, false for active).
///     - Ex: `{ "Servo": { "Rest": true } }`
/// - `PanTilt(u8, u8)`: Simultaneously pan and tilt the servo to specified angles.
///     - Ex: `{ "Servo": { "PanTilt": [30, 45] } }`
#[derive(Copy, Clone, fmt::Debug, serde::Serialize, serde::Deserialize)]
pub enum ServoCommand {
    Pan(u8),
    Tilt(u8),
    Rest(bool),
    PanTilt(u8, u8),
}

/// Servo Pair
///
/// A pair of servo motors for pan and tilt control.
///
/// This struct encapsulates two servos: one for panning and one for tilting.
/// It is designed to control two axes of motion, typically for camera or sensor stabilization.
///
/// # Type Parameters
/// - `Pan`: The PWM pin type used to control the pan servo.
/// - `Tilt`: The PWM pin type used to control the tilt servo.
///
/// # Fields
/// - `pan`: The servo motor responsible for panning.
/// - `tilt`: The servo motor responsible for tilting.
///
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

    /// Helper function to convert angle (0 to 180) to pwm signal based on 14bit
    fn pwm_value(angle: u8) -> u16 {409 * ((2048-409) / 180 * u16::from(angle))}

    /// Process websocket commands
    fn process(&mut self, command: ServoCommand) -> Result<(), Self::Error>;
}

impl<PwmError: fmt::Debug, Pan: PwmPin<Error = PwmError>, Tilt: PwmPin<Error = PwmError>> Servo
    for ServoPair<Pan, Tilt>
{
    type Error = PwmError;

    fn move_to(&mut self, duty_x: u16, duty_y: u16) -> Result<(), Self::Error> {
        match (
            self.pan.set_duty_cycle(duty_x),
            self.tilt.set_duty_cycle(duty_y),
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
            ServoCommand::PanTilt(x, y) => self.move_to(Self::pwm_value(x), Self::pwm_value(y)),
        }
    }
}
