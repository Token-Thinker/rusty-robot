#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]
#![feature(type_alias_impl_trait)]

pub mod hardware;
pub mod prelude;

use esp_println::println;
use hardware::{servo_ctrl::*, motor_ctrl::*};
#[allow(unused_imports)]
use prelude::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

#[async_task]
async fn motor_control_task(mut pin: impl Motor + 'static) {
    loop {
        pin.process_command().await.unwrap();
        Timer::after(Duration::from_millis(10)).await;
    }

    //Test loop
/*    loop{
        pin.launch().await.unwrap();
    }*/
}

#[async_task]
async fn servo_control_task(mut servos: impl PanTiltServoCtrl + 'static) {

    loop {
        servos.process_servo_command().await.unwrap();
        Timer::after(Duration::from_millis(10)).await;
    }

    //Test loop
/*    fn pwm_value(angle: u8) -> u16 { 409 + ((2048 - 409) / 180 * u16::from(angle))}

    loop {
        // Go forward
        for angle in 0..=180{
            servos.move_to(pwm_value(angle),pwm_value(angle))
                .expect("PanTilt command failed");
            Timer::after(Duration::from_millis(10)).await;
        }

        // Go backward
        for angle in (0..=180).rev() {
            servos.move_to(pwm_value(angle),pwm_value(angle))
                .expect("PanTilt command failed");
            Timer::after(Duration::from_millis(10)).await;
        }
    }*/
}

#[cfg(all(target_os = "none", target_arch = "xtensa", target_vendor = "unknown"))]
#[main]
async fn main(_spawner: Spawner) {
    use core::fmt::Write;

    init_heap();

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // initialize emabassy
    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    //initialize pins
    let pin = io.pins.gpio4.into_push_pull_output();
    let pan_pin = io.pins.gpio12.into_push_pull_output();
    let tilt_pin = io.pins.gpio14.into_push_pull_output();

    //initialize ledc
    let mut ledc = make_static!(LEDC::new(peripherals.LEDC, make_static!(clocks)));
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    //initialize timer
    let lstimer1 = make_static!(ledc.get_timer::<LowSpeed>(timer::Number::Timer1));

    lstimer1
    .configure(timer::config::Config {
        duty: timer::config::Duty::Duty14Bit,
        clock_source: timer::LSClockSource::APBClk,
        frequency: 50u32.Hz(),
    })
    .unwrap();

    //configure channel
    let mut channel1 = ledc.get_channel(channel::Number::Channel1, pan_pin);
    let mut channel2 = ledc.get_channel(channel::Number::Channel2, tilt_pin);

    channel1
        .configure(channel::config::Config {
            timer: lstimer1,
            duty_pct: 10,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();

    channel2
        .configure(channel::config::Config {
            timer: lstimer1,
            duty_pct: 10,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();

    let servos = PanTiltServos::new(channel1, channel2);

    _spawner.spawn(motor_control_task(pin)).ok();
    _spawner.spawn(servo_control_task(servos)).ok();

}