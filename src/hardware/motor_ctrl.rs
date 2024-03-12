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
//! async fn motor_control_task<PIN: OutputPin>(mut motor: MotorImpl<PIN>) {
//!     loop {
//!         match motor.process_command().await {
//!             Ok(()) => (), 
//!             Err(err) => { 
//!                 error!("Error during motor control: {:?}", err);
//!                 }
//!             }
//!         Timer::after(Duration::from_millis(10)).await; // Adjust polling interval as needed
//!     }
//! }
//! ```

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embedded_hal::digital::OutputPin;
use embassy_time::{Timer, Duration};

pub static MOTOR_CTRL_SIGNAL: Signal<CriticalSectionRawMutex, MotorCommand> = Signal::new();

// Motor Command
#[derive(Debug)]
pub enum MotorCommand {
    On,
    Off,
    Launch,
}

// Custom Error Type
#[derive(Debug)]
enum Error {
    PinError,
}

// Motor Trait - Represents the core functions  
trait Motor {
    async fn on(&mut self) -> Result<(), Error>;
    async fn off(&mut self) -> Result<(), Error>;
    async fn launch(&mut self) -> Result<(), Error>;
    async fn process_command(&mut self) -> Result<(), Error>;
}

// HAL Mapping for ESP32 Xtensa
#[cfg(all(target_os = "none", target_arch = "xtensa", target_vendor = "unknown"))]
mod esp_hal_mapping {
    use super::OutputPin;
    use hal::gpio::{GpioPin as EspOutputPin,  PushPull, Output};
    // Implement the embedded-hal OutputPin trait for ESP-IDF's OutputPin type
    impl<const GPIONUM: u8> OutputPin for EspOutputPin<Output<PushPull>, GPIONUM>
      {
        
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.set_low().unwrap();
            Ok(()) 
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.set_low().unwrap();
            Ok(())
        }
        
        fn set_state(&mut self, state: embedded_hal::digital::PinState) -> Result<(), Self::Error> {
            match state {
                embedded_hal::digital::PinState::Low => self.set_low(),
                embedded_hal::digital::PinState::High => self.set_high(),
            }
        }
    }
}

// HAL Mapping for RP2040
#[cfg(target_arch = "arm")]
mod rp2040_hal_mapping {
    use super::OutputPin;
    use rp2040_hal::gpio::Pin as Rp2040Pin;
    use rp2040_hal::gpio::functions::GpioFunction;

    impl OutputPin for Rp2040Pin<GpioFunction::PIO0> {
        type Error = rp2040_hal::gpio::Error; 

        //todo!()
    }
}

// Concrete Motor Implementations using a specific HAL & Embassy
pub struct MotorImpl<PIN: OutputPin> {
    pin: PIN,
}

impl<PIN: OutputPin> Motor for MotorImpl<PIN> {
    fn on(&mut self) -> Result<(), Error> {
        self.pin.set_high().map_err(|_| Error::PinError)
    }
    
    fn off(&mut self) -> Result<(), Error> {
        self.pin.set_low().map_err(|_| Error::PinError)
    }
    
    async fn launch(&mut self) -> Result<(), Error> {
        for _ in 0..100 {
            self.pin.toggle().map_err(|_| Error::PinError)?;
            Timer::after(Duration::from_millis(1)).await;
            self.pin.toggle().map_err(|_| Error::PinError)?;
            Timer::after(Duration::from_millis(1)).await;
        }
    
        Ok(())
    }

    async fn process_command(&mut self) -> Result<(), Error> {
        if let Some(command) = MOTOR_CTRL_SIGNAL.try_recv() {
           match command {
                MotorCommand::On => self.on(),
                MotorCommand::Off => self.off(),
                MotorCommand::Launch => self.launch().await,  
           }
        }
        Ok(())
    }
}

