#![no_std]
#![no_main]
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

#[async_task]
async fn servo_control_task(pan_pin: impl Motor + 'static, tilt_pin: impl Motor + 'static) {
    let mut servo_system = ServoSystem::new_servo_system(pan_pin, tilt_pin);

    loop {
        // Loop to pan back and forth
        for angle in [0, 180].iter() {
            servo_system.process_command(ServoCommand::Pan(*angle as i32))
                .expect("Pan command failed");
            Timer::after(Duration::from_millis(1000)).await; // Non-blocking delay
        }

        // Loop to tilt back and forth
        for angle in [0, 180].iter() {
            servo_system.process_command(ServoCommand::Tilt(*angle as i32))
                .expect("Tilt command failed");
            Timer::after(Duration::from_millis(1000)).await; // Non-blocking delay
        }
    }
}

#[cfg(all(target_os = "none", target_arch = "xtensa", target_vendor = "unknown"))]
#[main]
async fn main(_spawner: Spawner) {
    init_heap();

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX); 
    
    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    let pin = io.pins.gpio4.into_push_pull_output();

    _spawner.spawn(motor_control_task(pin)).ok();

}