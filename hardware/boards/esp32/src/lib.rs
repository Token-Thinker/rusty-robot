#![no_std]
#![allow(unused_qualifications)]
#![feature(type_alias_impl_trait)]


extern crate alloc;

use core::mem::MaybeUninit;

use esp_alloc::EspHeap;
use hal::{
    clock::ClockControl,
    gpio::{AnyOutput, GpioPin, Io, Level, OutputPin},
    ledc::{{channel::{self, ChannelIFace}}, {timer::{self, TimerIFace}}, Ledc, LSGlobalClkSource, LowSpeed},
    peripherals::Peripherals,
    rng::Rng,
    timer::timg::TimerGroup,
    system::SystemControl,
    prelude::_fugit_RateExtU32,
};
use esp_wifi::{
    initialize,
    EspWifiInitFor,
    wifi::{WifiStaDevice, WifiDevice, WifiController},
};

use seq_macro::seq;


pub struct Board {
    pub wifi_driver: WifiDevice<'static, WifiStaDevice>,
    pub wifi_controller: Option<WifiController<'static>>,
    pub flywheels: AnyOutput<'static>,
    pub loader: AnyOutput<'static>,
    pub pan: channel,
    pub tilt: channel,
}

impl Board {
    #[global_allocator]
    fn heap() {
        static ALLOCATOR: EspHeap = EspHeap::empty();
        const HEAP_SIZE: usize = 32 * 1024;
        static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();
        unsafe {
            ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
        }
    }

    /*        seq!(P in 0..=20 {
                fn get_gpio_pin<const P: u8>(io: &mut Io, pin: u8) -> Option<GpioPin<P>> {
                    match pin {
                        #(P => Some(io.pins.gpioP()),)*
                        _ => None,
                    }
                }
            });
    */
    pub fn init(&self) -> Board {
        Self::heap();

        let peripherals = Peripherals::take();
        let system = SystemControl::new(peripherals.SYSTEM);
        let clocks = ClockControl::max(system.clock_control).freeze();
        let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

        // Network Services Configurations
        let timer = TimerGroup::new(peripherals.TIMG1, &clocks, None).timer0;
        let init = initialize(
            EspWifiInitFor::Wifi,
            timer,
            Rng::new(peripherals.RNG),
            peripherals.RADIO_CLK,
            &clocks,
        ).unwrap();
        let wifi = peripherals.WIFI;
        let (wifi_driver, wifi_controller) =
            esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

        // initialize pins
        /*            let flywheels =  AnyOutput::new(Self::get_gpio_pin(&mut io, flywheels), Level::Low);
                    let loader =  AnyOutput::new(Self::get_gpio_pin(&mut io, loader), Level::Low);
                    let pan = Self::get_gpio_pin(&mut io, pan);
                    let tilt = Self::get_gpio_pin(&mut io, tilt);*/

        let flywheels = AnyOutput::new(io.pins.gpio4, Level::Low);
        let loader = AnyOutput::new(io.pins.gpio5, Level::Low);
        let pan = io.pins.gpio10;
        let tilt = io.pins.gpio11;

        // initialize ledc
        let mut ledc = Ledc::new(peripherals.LEDC, &(clocks));
        ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

        // initialize timer
        let mut lstimer1 = ledc.get_timer::<LowSpeed>(timer::Number::Timer1);

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
                timer: &lstimer1,
                duty_pct: 10,
                pin_config: channel::config::PinConfig::PushPull,
            })
            .unwrap();

        tchannel
            .configure(channel::config::Config {
                timer: &lstimer1,
                duty_pct: 10,
                pin_config: channel::config::PinConfig::PushPull,
            })
            .unwrap();

        Board {
            wifi_driver,
            wifi_controller: Option::from(wifi_controller),
            flywheels,
            loader,
            pan: (pchannel),
            tilt: (tchannel),
        }
    }
}