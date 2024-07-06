//! ## Motor Control Module
//!
//! Adds basic single-motor control functionality, including
//! configurable launch sequences to any type that implements
//! [OutputPin](embedded_hal::digital::OutputPin).
//!

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
pub enum MotorCommand {
    On,
    Off,
    Launch,
}

/// Motor Trait
///
/// TODO(mguerrier): documentation
#[allow(async_fn_in_trait)]
pub trait Motor {
    type Error: fmt::Debug;

    /// Turn the motor on
    fn on(&mut self) -> Result<(), Self::Error>;

    /// Turn the motor off
    fn off(&mut self) -> Result<(), Self::Error>;

    /// Execute a customizable launch sequence (e.g. rapid toggling for initialization)
    async fn launch(&mut self) -> Result<(), Self::Error>;

    /// Handle commands received via the global `MOTOR_CTRL_SIGNAL`
    async fn process(&mut self, command: MotorCommand) -> Result<(), Self::Error>;
}

// Concrete Motor Implementations using a specific HAL & Embassy
impl<T: OutputPin> Motor for T {
    type Error = T::Error;

    fn on(&mut self) -> Result<(), Self::Error> {
        self.set_high()
    }

    fn off(&mut self) -> Result<(), Self::Error> {
        self.set_low()
    }

    // TODO(mguerrier): configure launch sequence for smooth transition
    async fn launch(&mut self) -> Result<(), Self::Error> {
        for _ in 0..100 {
            self.set_high()?;
            Timer::after(Duration::from_millis(100)).await;
            self.set_low()?;
            Timer::after(Duration::from_millis(100)).await;
        }
        Ok(())
    }

    async fn process(&mut self, command: MotorCommand) -> Result<(), Self::Error> {
        match command {
            MotorCommand::On => self.on(),
            MotorCommand::Off => self.off(),
            MotorCommand::Launch => Ok(self.launch().await?),
        }
    }
}
