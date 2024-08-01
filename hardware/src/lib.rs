#![allow(unexpected_cfgs, unused_qualifications)]
#![cfg_attr(
    all(feature = "board", target_os = "none", target_vendor = "unknown"),
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
pub use tkr_hardware_board_local as board;

// RP2040 target
#[cfg(all(
    feature = "mcu",
    target_os = "none",
    target_arch = "arm",
    target_vendor = "unknown"
))]
pub use tkr_hardware_board_rp2040 as board;

pub use crate::{
    motor::{Motor, MotorCommand},
    servo::{Servo, ServoCommand, ServoPair},
};
#[cfg(feature = "mcu")]
pub mod mcu {
    use super::{board::MCU, Motor, Servo, ServoPair};
    use embassy_net::driver::Driver;

    pub trait MCUConfig {
        type WifiDriver: Driver;
        type Flywheels: Motor;
        type Loader: Motor;
        type Pan: Servo;
        type Tilt: Servo;

        fn components(&self) -> MCUComponents<Self>;
    }

    pub struct MCUComponents<'a, B: MCUConfig> {
        pub wifi_driver: &'a B::WifiDriver,
        pub flywheels: &'a B::Flywheels,
        pub loader: &'a B::Loader,
        pub pan: &'a B::Pan,
        pub tilt: &'a B::Tilt,
    }

    pub struct InitializedMCU<'a, B: MCUConfig> {
        pub components: MCUComponents<'a, B>,
        pub servos: ServoPair<&'a B::Pan, &'a B::Tilt>,
    }

    pub fn setup<B: MCUConfig>(
        mcu_config: &B,
    ) -> InitializedMCU<B> {
        let components = mcu_config.components();
        let servos = ServoPair::new(components.pan, components.tilt);
        InitializedMCU { components, servos }
    }

    pub fn init_mcu<'a>() -> InitializedMCU<'a, impl MCUConfig + 'a> {
        let mcu = MCU::init();
        setup(&mcu)
    }
}