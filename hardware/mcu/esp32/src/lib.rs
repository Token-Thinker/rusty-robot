#![no_std]
#![allow(unused_qualifications)]
#![feature(type_alias_impl_trait)]


extern crate alloc;

use core::mem::MaybeUninit;
use static_cell::{make_static, StaticCell};
use embassy_time::{Duration, Timer};


use esp_wifi::{
    initialize,
    wifi::{WifiController, WifiDevice, WifiStaDevice},
    EspWifiInitFor,
};
use esp_wifi::wifi::{ClientConfiguration, Configuration, WifiEvent, WifiState};
use hal::timer::{ErasedTimer, PeriodicTimer};
use hal::{
    clock::ClockControl,
    gpio::{AnyOutput, GpioPin, Io, Level},
    ledc::{
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
        LSGlobalClkSource, Ledc, LowSpeed,
    },
    peripherals::Peripherals,
    prelude::_fugit_RateExtU32,
    rng::Rng,
    system::SystemControl,
    timer::timg::TimerGroup,
};

pub use hal::prelude::main;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

pub struct MCU<WifiDriver, Flywheels, Loader, Pan, Tilt> {
    pub wifi_driver: WifiDriver,
    pub flywheels: Flywheels,
    pub loader: Loader,
    pub pan: Pan,
    pub tilt: Tilt,
}

static mut WIFI_MANAGER: Option<WifiManager> = None;


pub struct WifiManager{
    pub wifi_controller: WifiController<'static>
}


impl MCU<WifiDevice<'static, WifiStaDevice>, AnyOutput<'static>, AnyOutput<'static>, hal::ledc::channel::Channel<'static, LowSpeed, GpioPin<10>>, hal::ledc::channel::Channel<'static, LowSpeed, GpioPin<11>>> {
    pub fn init() -> Self {
        init_heap();

        let peripherals = Peripherals::take();
        let system = SystemControl::new(peripherals.SYSTEM);
        let clocks = ClockControl::max(system.clock_control).freeze();
        let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

        let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks, None);
        let timer0: ErasedTimer = timg0.timer0.into();
        let timer = PeriodicTimer::new(timer0);

        // Network Services Configurations
        let init = initialize(
            EspWifiInitFor::Wifi,
            timer,
            Rng::new(peripherals.RNG),
            peripherals.RADIO_CLK,
            &clocks,
        )
        .unwrap();
        let wifi = peripherals.WIFI;
        let (wifi_driver, wifi_controller) =
            esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();


        let flywheels = AnyOutput::new(io.pins.gpio4, Level::Low);
        let loader = AnyOutput::new(io.pins.gpio5, Level::Low);
        let pan = io.pins.gpio10;
        let tilt = io.pins.gpio11;

        // initialize ledc
        let ledc = make_static!(Ledc::new(peripherals.LEDC, make_static!(clocks)));
        ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

        // initialize timer
        let lstimer1 = make_static!(ledc.get_timer::<LowSpeed>(timer::Number::Timer1));

        lstimer1
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty14Bit,
                clock_source: timer::LSClockSource::APBClk,
                frequency: 50u32.Hz(),
            })
            .unwrap();

        // configure channels
        let mut pchannel = ledc.get_channel(channel::Number::Channel1, pan);
        let mut tchannel = ledc.get_channel(channel::Number::Channel2, tilt);

        pchannel
            .configure(channel::config::Config {
                timer: lstimer1,
                duty_pct: 10,
                pin_config: channel::config::PinConfig::PushPull,
            })
            .unwrap();

        tchannel
            .configure(channel::config::Config {
                timer: lstimer1,
                duty_pct: 10,
                pin_config: channel::config::PinConfig::PushPull,
            })
            .unwrap();

        let mcu = MCU {
            wifi_driver,
            flywheels,
            loader,
            pan: (pchannel),
            tilt: (tchannel),
        };

        let wifi_manager = WifiManager{
            wifi_controller,
        };

        unsafe {
            WIFI_MANAGER = Some(wifi_manager);
        }

        mcu
    }
}

const SSID: &str = "SSID";
const PASSWORD: &str = "PASSWORD";
#[embassy_executor::task]
pub async fn connection() {
    unsafe {
        if let Some(manager) = WIFI_MANAGER.as_mut() {
            let controller = &mut manager.wifi_controller;

            loop {
                match esp_wifi::wifi::get_wifi_state() {
                    WifiState::StaConnected => {
                        controller.wait_for_event(WifiEvent::StaDisconnected).await;
                        Timer::after(Duration::from_millis(5000)).await;
                    }
                    _ => {}
                }

                if !matches!(controller.is_started(), Ok(true)) {
                    let client_config = Configuration::Client(ClientConfiguration {
                        ssid: SSID.try_into().unwrap(),
                        password: PASSWORD.try_into().unwrap(),
                        ..Default::default()
                    });
                    controller.set_configuration(&client_config).unwrap();
                    controller.start().await.unwrap();
                }

                match controller.connect().await {
                    Ok(_) => (),
                    Err(e) => {
                        Timer::after(Duration::from_millis(5000)).await;
                    }
                }
            }
        }
    }
}
