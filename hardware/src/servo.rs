//! ## Servo Control Module
//!
//! Facilitates precise positioning of servos through angle conversion and command processing,
//! supporting pan and tilt functionalities for [SetDutyCycle](embedded_hal::pwm::SetDutyCycle;)
//!
use core::fmt;
use embedded_hal::pwm::{self, SetDutyCycle, Error, ErrorType};

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

/// Servo Error
#[derive(Debug)]
pub enum ServoError<P, T> {
    PanError(P),
    TiltError(T),
    BothErrors(P, T),
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
pub struct ServoPair<Pan: SetDutyCycle, Tilt: SetDutyCycle> {
    pub(crate) pan: Pan,
    pub(crate) tilt: Tilt,
}

impl<Pan: SetDutyCycle, Tilt: SetDutyCycle> ServoPair<Pan, Tilt> {
    /// Create a new `ServoPair` instance from the supplied pins
    pub fn new(pan: Pan, tilt: Tilt) -> Self {
        Self { pan, tilt }
    }
}

/// Servo Trait
///
/// This trait defines the fundamental operations that a servo should support.
/// It includes methods to move the servo to specified coordinates, convert
/// an angle to a PWM signal, and process commands received via WebSocket.
///
/// The `Servo` trait is designed to be implemented for various types of servos,
/// allowing for flexibility and extensibility in servo control implementations.
pub trait Servo {
    type Error: fmt::Debug;

    /// Move the servo pair to the specified coordinates
    ///
    /// This method is responsible for moving the servo to the given x and y coordinates.
    /// Implementations should ensure that the servo moves smoothly and accurately
    /// to the specified position.
    ///
    /// # Parameters
    ///
    /// * `x` - The x-coordinate to move the servo to.
    /// * `y` - The y-coordinate to move the servo to.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Returns `Ok(())` if the servo successfully moves to the
    /// specified coordinates, or an error of type `Self::Error` if the operation fails.
    fn move_to(&mut self, x: u16, y: u16) -> Result<(), Self::Error>;

    /// Helper function to convert angle (0 to 180) to PWM signal based on 14-bit resolution
    ///
    /// This function converts an angle in the range of 0 to 180 degrees to a corresponding
    /// PWM signal value. The conversion is based on a 14-bit resolution.
    ///
    /// # Parameters
    ///
    /// * `angle` - The angle in degrees to be converted to a PWM signal.
    ///
    /// # Returns
    ///
    /// * `u16` - The PWM signal value corresponding to the given angle.
    fn pwm_value(angle: u8) -> u16 {
        409 * ((2048 - 409) / 180 * u16::from(angle))
    }

    /// Process Commands
    ///
    /// This method processes commands sent to the servo. The `ServoCommand`
    /// parameter represents the specific command to be executed.
    ///
    /// # Parameters
    ///
    /// * `command` - A `ServoCommand` instance representing the command to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Returns `Ok(())` if the command is successfully processed,
    /// or an error of type `Self::Error` if the operation fails.
    fn process(&mut self, command: ServoCommand) -> Result<(), Self::Error>;
}

impl <P: SetDutyCycle, T: SetDutyCycle >Servo for ServoPair<P, T> {
    type Error = ServoError<P::Error, T::Error>;

    fn move_to(&mut self, x: u16, y: u16) -> Result<(), Self::Error> {
        match (self.pan.set_duty_cycle(x), self.tilt.set_duty_cycle(y)) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(pan_error), Ok(())) => Err(ServoError::PanError(pan_error)),
            (Ok(()), Err(tilt_error)) => Err(ServoError::TiltError(tilt_error)),
            (Err(pan_error), Err(tilt_error)) => Err(ServoError::BothErrors(pan_error, tilt_error)),
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