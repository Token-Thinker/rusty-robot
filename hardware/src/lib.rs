#![no_std]
#![allow(unexpected_cfgs, unused_qualifications)]
#![cfg_attr(
    all(feature = "mcu", target_os = "none", target_vendor = "unknown"),
    no_std
)]

//! ## Hardware Library
//!
//! This crate provides the interface for interacting with a robot's hardware components.
//! It includes submodules for specific hardware control:
//!
//! * **motor:** Functions for controlling a motor (On, Off, Launch).
//! * **servo:** Fine-grained servo control, including configuration, angle mapping, and smooth movement.

/// Motor Module
///
/// This module provides the implementation for motor control. It includes functions
/// to turn the motor on and off, as well as execute a launch sequence. The module
/// defines the `Motor` trait and `MotorCommand` enum for standardized motor operations.
pub mod motor;

/// Servo Module
///
/// This module offers detailed control over servo motors. It includes functionality
/// for configuring servos, mapping angles, and ensuring smooth movements. The module
/// defines the `Servo` trait, `ServoCommand` enum, and `ServoPair` struct for managing
/// servo operations.
pub mod servo;

// ESP32 target
#[cfg(all(
    feature = "mcu",
    target_os = "none",
    target_arch = "xtensa",
    target_vendor = "unknown"
))]
pub use tkr_hardware_mcu_esp32 as board;

// Local host target
#[cfg(all(
    feature = "mcu",
    not(target_os = "none"),
    not(target_vendor = "unknown")
))]
pub use tkr_hardware_mcu_local as board;

// RP2040 target
#[cfg(all(
    feature = "mcu",
    target_os = "none",
    target_arch = "arm",
    target_vendor = "unknown"
))]
pub use tkr_hardware_mcu_rp2040 as board;

pub use crate::{
    motor::{Motor, MotorCommand},
    servo::{Servo, ServoCommand, ServoPair},
};
#[cfg(feature = "mcu")]
pub mod mcu {
    use super::{board::MCU, Motor, ServoPair};
    use embassy_net::driver::Driver;
    use embedded_hal::pwm::SetDutyCycle;

    pub trait MCUConfig <WifiDriver: Driver, Flywheels: Motor, Loader: Motor, Pan: SetDutyCycle, Tilt: SetDutyCycle> {
        fn components(self) -> MCUComponents<WifiDriver, Flywheels, Loader, Pan, Tilt>;
    }

    pub struct MCUComponents<WifiDriver: Driver, Flywheels: Motor, Loader: Motor, Pan: SetDutyCycle, Tilt: SetDutyCycle> {
        pub wifi_driver: WifiDriver,
        pub flywheels: Flywheels,
        pub loader: Loader,
        pub servos: ServoPair<Pan, Tilt>,
    }

    impl <WifiDriver: Driver, Flywheels: Motor, Loader: Motor, Pan: SetDutyCycle, Tilt: SetDutyCycle>
    MCUConfig<WifiDriver, Flywheels, Loader, Pan, Tilt> for MCU<WifiDriver, Flywheels, Loader, Pan, Tilt> {
        fn components(self) -> MCUComponents<WifiDriver, Flywheels, Loader, Pan, Tilt> {
            MCUComponents{
                wifi_driver: self.wifi_driver,
                flywheels: self.flywheels,
                loader: self.loader,
                servos: ServoPair { pan: self.pan, tilt: self.tilt },
            }

        }
    }

    pub fn init_mcu() -> MCUComponents<
        impl Driver,
        impl Motor,
        impl Motor,
        impl SetDutyCycle,
        impl SetDutyCycle,
    > {
        let mcu = MCU::init();
        mcu.components()
    }
}