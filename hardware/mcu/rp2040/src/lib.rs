//Highly unstable - not tested at all

#![no_std]
#![no_main]

use panic_halt as _;
use rp2040_hal as hal;

use rp2040_hal::clocks::Clock;
use hal::{pac, pwm, gpio};

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;
const LOW: u16 = 0;
const HIGH: u16 = 25000;
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

pub struct MCU {
    //pub wifi_driver: todo,
    //pub wifi_controller: todo,
    pub flywheels:  gpio::Pin<gpio::bank0::Gpio8, gpio::FunctionSio<gpio::SioOutput>, gpio::PullDown>,
    pub loader: gpio::Pin<gpio::bank0::Gpio10, gpio::FunctionSio<gpio::SioOutput>, gpio::PullDown>,
    pub pan:  pwm::Channel<pwm::Slice<pwm::Pwm4, pwm::FreeRunning>, pwm::B>,
    pub tilt: pwm::Channel<pwm::Slice<pwm::Pwm2, pwm::FreeRunning>, pwm::A>,
}

impl MCU{
    pub fn init() -> MCU{
        // Grab our singleton objects
        let mut pac = pac::Peripherals::take().unwrap();
        let core = pac::CorePeripherals::take().unwrap();

        // Set up the watchdog driver - needed by the clock setup code
        let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

        // Configure the clocks
        //
        // The default is to generate a 125 MHz system clock
        let clocks = hal::clocks::init_clocks_and_plls(
            XTAL_FREQ_HZ,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        )
            .unwrap();

        // The single-cycle I/O block controls our GPIO pins
        let sio = hal::Sio::new(pac.SIO);

        // Set the pins up according to their function on this particular board
        let pins = hal::gpio::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        // Configure GPIO 25 as an output
        let mut flywheels = pins.gpio8.into_push_pull_output();
        let mut loader = pins.gpio10.into_push_pull_output();

        // Init PWMs
        let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);

        // Configure PWM2
        let mut pwm0 = pwm_slices.pwm2;
        pwm0.set_ph_correct();
        pwm0.enable();

        // Configure PWM4
        let mut pwm = pwm_slices.pwm4;
        pwm.set_ph_correct();
        pwm.enable();

        // Output channel B on PWM4 to GPIO 25
        let mut pan = pwm.channel_b;
        pan.output_to(pins.gpio25);

        // Output channel A on PWM2 to GPIO 25
        let mut tilt = pwm0.channel_a;
        tilt.output_to(pins.gpio4);

        MCU {
            flywheels,
            loader,
            pan,
            tilt,
        }

    }
}
