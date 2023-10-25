#![allow(unused_imports)]

//! ## TKR's `prelude` Module
//!
//! This "module" actually contains several "setup" type
//! module blocks differentiated by intended target. Each
//! target-specific block pulls in the necessary dependencies
//! for a given compilation target and handles the necessary
//! setup to present a uniform interface to the project's
//! "common" dependencies.

#[cfg(all(not(target_os = "none"), not(target_vendor = "unknown")))]
pub use default_prelude::*;
#[cfg(all(target_os = "none", target_arch = "xtensa", target_vendor = "unknown"))]
pub use esp32_prelude::*;
#[cfg(all(target_os = "none", target_arch = "arm", target_vendor = "unknown"))]
pub use rp2040_prelude::*;

#[cfg(all(not(target_os = "none"), not(target_vendor = "unknown")))]
pub mod default_prelude {
    panic!();
}

#[cfg(all(target_os = "none", target_arch = "arm", target_vendor = "unknown"))]
pub mod rp2040_prelude {
    panic!();
}

#[cfg(all(target_os = "none", target_arch = "xtensa", target_vendor = "unknown"))]
pub mod esp32_prelude {
    #[allow(clippy::single_component_path_imports)]
    pub use esp_backtrace;

    pub use esp_println::logger;
    pub use hal::{
        clock::ClockControl,
        embassy,
        embassy::executor::Executor,
        gpio,
        gpio::{PushPull, Output, GpioPin},
        ledc::{
            channel::{self, ChannelIFace},
            timer::{self, TimerIFace},
            HighSpeed,
            LEDC,
        },
        peripherals::Peripherals,
        prelude::*,
        timer::TimerGroup, // Rng,
    };
}
