//! ## Hardware Module
//!
//! This module provides the interface for interacting with the application's hardware components. It includes submodules for specific hardware control:
//!
//! * **motor_control:** Functions for controlling a motor (On, Off, Launch).
//! * **servo_control:**  Fine-grained servo control, including configuration, angle mapping, and smooth movement. 

pub mod servo_ctrl;
pub mod motor_ctrl;
