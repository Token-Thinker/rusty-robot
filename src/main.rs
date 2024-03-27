#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod hardware;
pub mod prelude;

use hardware::motor_ctrl::*;
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