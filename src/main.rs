#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]
#![feature(type_alias_impl_trait)]

pub mod hardware;
pub mod prelude;

use hardware::{servo_ctrl::*,motor_ctrl::*};
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
        //pin.process_command().await.unwrap();
        pin.launch().await.unwrap();
        //Timer::after(Duration::from_millis(10)).await;
        
    }
}

//PanTiltServo<impl PwmPin + 'static, impl PwmPin + 'static>


#[async_task]
async fn servo_control_task(mut servos: impl PanTiltServoCtrl + 'static) {
    loop {
        // Loop to move in a pattern
        for angle in [0, 179].iter() {
            // Combining pan and tilt movements into a single command
            servos.process_servo_command(ServoCommand::PanTilt(*angle as i32, *angle as i32))
                .expect("PanTilt command failed");
            Timer::after(Duration::from_millis(10000)).await;
        }
    }
}

#[cfg(all(target_os = "none", target_arch = "xtensa", target_vendor = "unknown"))]
#[main]
async fn main(_spawner: Spawner) {
    init_heap();

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX); 

    // initialize emabssy
    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    //initialize pins
    let pin = io.pins.gpio4.into_push_pull_output();
    let pan_pin = io.pins.gpio12;
    let tilt_pin = io.pins.gpio14;


    // initialize peripheral
    //#[cfg(feature = "esp32h2")]
    //let clock_cfg = PeripheralClockConfig::with_frequency(&clocks, 40.MHz()).unwrap();
    //#[cfg(not(feature = "esp32h2"))]
    let clock_cfg = PeripheralClockConfig::with_frequency(&clocks, 32u32.MHz()).unwrap();
    
    let mut mcpwm = MCPWM::new(peripherals.MCPWM0, clock_cfg);
    
    // connect operator0 to timer0
    mcpwm.operator0.set_timer(&mcpwm.timer0);

    // connect operator0 to pan_pin
    let pwm_pan_pin = mcpwm
    .operator0
    .with_pin_a(pan_pin, PwmPinConfig::UP_ACTIVE_HIGH);
    
    // connect operator1 to titl_pin
    let pwm_tilt_pin = mcpwm
    .operator1
    .with_pin_a(tilt_pin, PwmPinConfig::UP_ACTIVE_HIGH);

    let servos = PanTiltServos::new(pwm_pan_pin, pwm_tilt_pin);

    _spawner.spawn(motor_control_task(pin)).ok();
    _spawner.spawn(servo_control_task(servos)).ok();


}