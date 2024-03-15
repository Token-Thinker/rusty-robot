//! ## Motor Control Module
//!
//! Provides basic control functions for a single motor, including the ability to turn it 
//! on/off and configure custom launch sequences.
//!
//! **Key Functions**
//!
//! * `on()`: Turns the motor on.
//! * `off()`: Turns the motor off.
//! * `launch()`: Executes a customizable launch sequence (e.g., rapid toggling for initialization).
//! * `process_command()`: Handles commands received via the `MOTOR_CTRL_SIGNAL`.
//!
//! **Error Handling**
//! * **PinError:** Indicates a generic error occurred when interacting with the GPIO pin. 
//!   Consider adding more specific error types for finer-grained error handling if needed.
//!
//! **Usage**
//! ```rust
//! 
//! use hardware::motor_ctrl::*
//! 
//! async fn motor_control_task<PIN: Motor>(mut pin: PIN) {
//!     loop {
//!         match pin.process_command().await {
//!             Ok(()) => (), 
//!             Err(err) => { 
//!                 todo!()
//!                 }
//!             }
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

// Custom Error Type
#[derive(fmt::Debug)]
#[allow(missing_docs)]
pub enum Error<> {
    PinError,
}

// Motor Trait
pub trait Motor {
    fn on(&mut self) -> Result<(), Error>;
    fn off(&mut self) -> Result<(), Error>;
    async fn launch(&mut self) -> Result<(), Error>;
    async fn process_command(&mut self) -> Result<(), Error>;
}
// Concrete Motor Implementations using a specific HAL & Embassy

impl<T: OutputPin> Motor for T {
    fn on(&mut self) -> Result<(), Error> {
        self.set_high().map_err(|_| Error::PinError)
    }
    
    fn off(&mut self) -> Result<(), Error> {
        self.set_low().map_err(|_| Error::PinError)
    }
    
    async fn launch(&mut self) -> Result<(), Error> {
        for _ in 0..100 {
            self.set_high().map_err(|_| Error::PinError);
            Timer::after(Duration::from_millis(1)).await;
            self.set_low().map_err(|_| Error::PinError);
            Timer::after(Duration::from_millis(1)).await;
        }
    
        Ok(())
    }

    async fn process_command(&mut self) -> Result<(), Error> {
        let command = MOTOR_CTRL_SIGNAL.wait().await;
        {
           match command {
                MotorCommand::On => self.on(),
                MotorCommand::Off => self.off(),
                MotorCommand::Launch => Ok(self.launch().await?),  
           };
        }
        Ok(())
    }
}

