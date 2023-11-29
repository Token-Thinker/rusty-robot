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
    pub use embedded_svc::wifi::{AccessPointConfiguration, ClientConfiguration, Configuration, Wifi};
    pub use static_cell::{make_static, StaticCell};

    pub use picoserve::{Router, routing::get, response::IntoResponse, extract::{State, Form}};


    pub use embassy_sync::{channel::{Channel, Receiver, Sender},blocking_mutex::raw::{CriticalSectionRawMutex,NoopRawMutex}, signal::Signal};
    pub use embassy_executor::Spawner;
    pub use embassy_time::{Duration, Ticker, Timer};
    
    pub use embassy_net::tcp::TcpSocket;
    pub use embassy_net::{
        Config, IpListenEndpoint, Ipv4Address, Ipv4Cidr, Stack, StackResources, StaticConfigV4,
    };

    pub use esp_backtrace;
    pub use esp_println::{logger, print, println};

    pub use esp_wifi::{initialize, EspWifiInitFor};
    pub use esp_wifi::wifi::{
        self, WifiApDevice, WifiController, WifiDevice, WifiEvent, WifiStaDevice, WifiState, WifiError
    };

    pub use hal::{
        clock::{ClockControl,Clocks},
        embassy::{self, executor::{Executor, FromCpu1, FromCpu2, InterruptExecutor}},
        gpio::{self, PushPull, Output, GpioPin},
        ledc::{
            channel::{self, ChannelIFace},
            timer::{self, TimerIFace},
            HighSpeed,
            LEDC,
        },
        peripherals::Peripherals,
        prelude::*,
        timer::TimerGroup, // Rng,
        cpu_control::{CpuControl, Stack as hal_stack},
        interrupt::Priority,
        Rng,
        get_core
    };
}
