#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod hardware;
pub mod prelude;

use hardware::motor_ctrl::*;
#[allow(unused_imports)]
use prelude::*;
use embassy_executor::{task as async_task, Spawner};

use core::mem::MaybeUninit;

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
        pin.process_command().await.map_err(|error| todo!());
        Timer::after(Duration::from_millis(10)).await; // Adjust polling interval as needed
    }
}



#[main]
async fn main(_spawner: Spawner) {
    init_heap();

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX); 
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0);

    let pin = io.pins.gpio4.into_push_pull_output();

    _spawner.spawn(motor_control_task(pin)).ok();

}