#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]
#![feature(type_alias_impl_trait)]

use comms::{messages::command_router, server::run as websocket_server};
use embassy_executor::Spawner;
use embassy_net::{driver::Driver, Config, Ipv4Address, Ipv4Cidr, Stack, StackResources};
use esp_hal::entry;
use hardware::mcu::{connection, init_mcu, main};
use static_cell::{make_static, StaticCell};

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<impl Driver>) -> ! { stack.run().await }

#[main]
async fn main(spawner: Spawner)
{
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

    tracing::info!("Starting WebSocket comms");

    // Run the WebSocket comms
    websocket_server(
        0,    // ID for the WebSocket comms instance
        8000, // Port number
        stack, None,
    )
    .await;

    spawner.spawn(command_router()).unwrap();
}
