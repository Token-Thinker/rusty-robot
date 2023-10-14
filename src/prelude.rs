#![allow(unused_imports)]

//! ## TKR's `prelude` Module
//!
//! This "module" actually contains several "setup" type
//! module blocks differentiated by intended target. Each
//! target-specific block pulls in the necessary dependencies
//! for a given compilation target and handles the necessary
//! setup to present a uniform interface to the project's
//! "common" dependencies.

#[cfg(not(target_os = "none"))]
pub(crate) use default_prelude::*;
#[cfg(target = "xtensa-esp32-none-elf")]
pub(crate) use esp32_prelude::*;
#[cfg(target = "thumbv6m-none-eabi")]
pub(crate) use rp2040_prelude::*;

#[cfg(not(target_os = "none"))]
pub(crate) mod default_prelude {
    pub use core::prelude::*;
}

#[cfg(target = "thumbv6m-none-eabi")]
pub(crate) mod rp2040_prelude {
    pub use core::prelude::*;
}

#[cfg(target = "xtensa-esp32-none-elf")]
pub(crate) mod esp32_prelude {
    #[allow(clippy::single_component_path_imports)]
    pub use esp_backtrace;

    pub use esp_println::logger;
    pub use hal::{
        clock::ClockControl,
        embassy,
        embassy::executor::Executor,
        gpio,
        peripherals::Peripherals,
        prelude::*,
        timer::TimerGroup, // Rng,
    };
}
