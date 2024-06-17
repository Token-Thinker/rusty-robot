#![allow(unexpected_cfgs, unused_qualifications)]
#![cfg_attr(
    all(feature = "board", target_os = "none", target_vendor = "unknown"),
    no_std
)]

//! ## Hardware
//!
//! This crate provides the interface for interacting with a robot's hardware components.
//! It includes submodules for specific hardware control:
//!
//! * **motor:** Functions for controlling a motor (On, Off, Launch).
//! * **servo:**  Fine-grained servo control, including configuration, angle mapping, and smooth movement.

pub mod motor;
pub mod servo;

// ESP32 target
#[cfg(all(
    feature = "board",
    target_os = "none",
    target_arch = "xtensa",
    target_vendor = "unknown"
))]
pub use tkr_hardware_board_esp32 as board;
// Local host target
#[cfg(all(
    feature = "board",
    not(target_os = "none"),
    not(target_vendor = "unknown")
))]
pub use tkr_hardware_board_local as board;
// RP2040 target
#[cfg(all(
    feature = "board",
    target_os = "none",
    target_arch = "arm",
    target_vendor = "unknown"
))]
pub use tkr_hardware_board_rp2040 as board;

pub use crate::{
    motor::{Motor, MotorCommand},
    servo::{Servo, ServoCommand, ServoPair},
};
