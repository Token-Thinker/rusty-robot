//! ## Motor Control Module
//!
//! Adds basic single-motor control functionality, including
//! configurable launch sequences to any type that implements
//! [OutputPin](embedded_hal::digital::OutputPin).
//!
//! ### Example Usage
//! ```rust
//! 
//! async fn motor_control_task<Pin: Motor>(&mut pin: Pin) {
//!     loop {
//!         pin.process_command().await.map_err(|error| todo!())?;
//!         Timer::after(Duration::from_millis(10)).await; // Adjust polling interval as needed
//!     }
//! }
//! ```

use core::fmt;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embedded_hal_1::digital::OutputPin;
use embassy_time::{Timer, Duration};

pub static MOTOR_CTRL_SIGNAL: Signal<CriticalSectionRawMutex, MotorCommand> = Signal::new();

// Motor Command
#[derive(fmt::Debug)]
pub enum MotorCommand {
    On,
    Off,
    Launch,
}


// Motor Trait
pub trait Motor {

    type Error;

    /// Turn the motor on
    fn on(&mut self) -> Result<(), Self::Error>;

    /// Turn the motor off
    fn off(&mut self) -> Result<(), Self::Error>;

    /// Execute a customizable launch sequence (e.g. rapid toggling for initialization)
    async fn launch(&mut self) -> Result<(), Self::Error>;

    /// Handle commands received via the global `MOTOR_CTRL_SIGNAL`
    async fn process_command(&mut self) -> Result<(), Self::Error>;
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

    async fn launch(&mut self) -> Result<(), Self::Error> {
        for _ in 0..100 {
            self.set_high()?;
            Timer::after(Duration::from_millis(1)).await;
            self.set_low()?;
            Timer::after(Duration::from_millis(1)).await;
        }
    
        Ok(())
    }

    async fn process_command(&mut self) -> Result<(), Self::Error> {
        match MOTOR_CTRL_SIGNAL.wait().await {
            MotorCommand::On => self.on(),
            MotorCommand::Off => self.off(),
            MotorCommand::Launch => Ok(self.launch().await?),
        }
    }
}
