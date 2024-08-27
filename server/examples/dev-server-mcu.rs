#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]
#![feature(type_alias_impl_trait)]

use static_cell::{make_static, StaticCell};

use embassy_executor::{Spawner};
use esp_hal::entry;

use embassy_net::{Config, Ipv4Address, Ipv4Cidr, Stack, StackResources, driver::Driver};

use hardware::mcu::{init_mcu, connection,main};
use tkr_server::{messages::command_router, server::run as websocket_server};

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<impl Driver>) -> ! {
    stack.run().await
}

#[main]
async fn main(spawner: Spawner) {

    let mcu = init_mcu();
    let mut device = mcu.wifi_driver;

    // Init network device
    let device = device;

    // Choose between dhcp or static ip
    let config = Config::dhcpv4(Default::default());

    // Generate random seed
    let seed = 1234;

    // Init network stack
    let stack = &*make_static!(Stack::new(
        device,
        config,
        make_static!(StackResources::<3>::new()),
        seed,
    ));

    // Launch network task
    spawner.spawn(net_task(stack)).unwrap();
    spawner.spawn(connection()).unwrap();

    tracing::info!("Starting WebSocket server");

    // Run the WebSocket server
    websocket_server(
        0,    // ID for the WebSocket server instance
        8000, // Port number
        stack, None,
    ).await;

    spawner.spawn(command_router()).unwrap();
}
