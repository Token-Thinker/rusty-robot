//! ## Motor Control Module
//!
//! Adds basic single-motor control functionality, including
//! configurable launch sequences to any type that implements
//! [OutputPin](embedded_hal::digital::OutputPin).

use core::fmt;

use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;

/// Motor Command
///
/// Variants:
/// - `On`: Turn the motor on.
///   - Ex: `{ "Motor": "On" }`
/// - `Off`: Turn the motor off.
///   - Ex: `{ "Motor": "Off" }`
/// - `Launch`: Launch the motor with a predefined sequence.
///   - Ex: `{ "Motor": "Launch" }`
#[derive(Copy, Clone, fmt::Debug, serde::Serialize, serde::Deserialize)]
pub enum MotorCommand
{
    On,
    Off,
    Launch,
}

/// Motor Trait
///
/// This trait defines the fundamental operations that a motor should support.
/// It includes methods to turn the motor on and off, execute a customizable
/// launch sequence, and process commands received via a global control signal.
///
/// The `Motor` trait is designed to be implemented for various types of motors,
/// allowing for flexibility and extensibility in motor control implementations.
#[allow(async_fn_in_trait)]
pub trait Motor
{
    type Error: fmt::Debug;

    /// Turn the motor on
    ///
    /// This method is responsible for powering on the motor. Implementations
    /// should ensure that any necessary initialization steps are performed
    /// when this method is called.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Returns `Ok(())` if the motor is
    ///   successfully turned on,
    ///   or an error of type `Self::Error` if the operation fails.
    fn on(&mut self) -> Result<(), Self::Error>;

    /// Turn the motor off
    ///
    /// This method is responsible for powering off the motor. Implementations
    /// should ensure that any necessary shutdown steps are performed
    /// when this method is called.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Returns `Ok(())` if the motor is
    ///   successfully turned off,
    ///   or an error of type `Self::Error` if the operation fails.
    fn off(&mut self) -> Result<(), Self::Error>;

    /// Execute a customizable launch sequence
    ///
    /// This asynchronous method allows for the execution of a launch sequence,
    /// which could involve rapid toggling or other initialization routines
    /// specific to the motor being controlled.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Returns `Ok(())` if the launch sequence is
    ///   successfully executed,
    ///   or an error of type `Self::Error` if the operation fails.
    async fn launch(&mut self) -> Result<(), Self::Error>;

    /// Process Commands
    ///
    /// This method processes commands sent to the servo. The `MotorCommand`
    /// parameter represents the specific command to be executed.
    ///
    /// # Parameters
    ///
    /// * `command` - A `MotorCommand` instance representing the command to be
    ///   processed.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Returns `Ok(())` if the command is
    ///   successfully processed,
    ///   or an error of type `Self::Error` if the operation fails.
    async fn process(
        &mut self,
        command: MotorCommand,
    ) -> Result<(), Self::Error>;
}

impl<T: OutputPin> Motor for T
{
    type Error = T::Error;

    fn on(&mut self) -> Result<(), Self::Error> { self.set_high() }

    fn off(&mut self) -> Result<(), Self::Error> { self.set_low() }

    // TODO(mguerrier): configure launch sequence for smooth transition
    async fn launch(&mut self) -> Result<(), Self::Error>
    {
        for _ in 0..100 {
            self.set_high()?;
            Timer::after(Duration::from_millis(100)).await;
            self.set_low()?;
            Timer::after(Duration::from_millis(100)).await;
        }
        Ok(())
    }

    async fn process(
        &mut self,
        command: MotorCommand,
    ) -> Result<(), Self::Error>
    {
        match command {
            MotorCommand::On => self.on(),
            MotorCommand::Off => self.off(),
            MotorCommand::Launch => Ok(self.launch().await?),
        }
    }
}
