//! ## TKR's `Network Access` Module
//!
//! This "module" contains embassy async tasks for connecting
//! to both STA and AP at the same time. The intent is for the 
//! end user to be able to access the ESP32 front-end either by
//! perferable connecting to the local wifi or by AP.


#[allow(unused_imports)]

use crate::prelude::*;
extern crate alloc;

use picoserve::{Router, routing::get, response::IntoResponse};


const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");


struct EmbassyTimer;

impl picoserve::Timer for EmbassyTimer {
    type Duration = embassy_time::Duration;
    type TimeoutError = embassy_time::TimeoutError;

    async fn run_with_timeout<F: core::future::Future>(
        &mut self,
        duration: Self::Duration,
        future: F,
    ) -> Result<F::Output, Self::TimeoutError> {
        embassy_time::with_timeout(duration, future).await
    }
}

async fn get_site() -> impl IntoResponse {
    (
        [("Content-Type", "text/html; charset=utf-8")],
        "<html>\
            <body>\
                <h1>Hello Rust!</h1>\
            </body>\
        </html>\r\n"
    )
}

#[embassy_executor::task]
pub async fn web_task(
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    config: &'static picoserve::Config<Duration>,
    //sender: Sender<'static, NoopRawMutex, MoveCommand,QUEUE_SIZE>
) -> ! {
    let mut rx_buffer = [0; 1536];
    let mut tx_buffer = [0; 1536];

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            println!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    loop {
        let mut socket = embassy_net::tcp::TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

        log::info!("Listening on TCP:8080...");
        if let Err(e) = socket.accept(8080).await {
            log::warn!("accept error: {:?}", e);
            continue;
        }

        log::info!(
            "Received connection from {:?}",
            socket.remote_endpoint()
        );

        let (socket_rx, socket_tx) = socket.split();

        let app = Router::new()
            .route("/", get(get_site))
        ;

        match picoserve::serve(
            &app,
            EmbassyTimer,
            config,
            &mut [0; 2048],
            socket_rx,
            socket_tx,
        )
        .await
        {
            Ok(handled_requests_count) => {
                log::info!(
                    "{handled_requests_count} requests handled from {:?}",
                    socket.remote_endpoint()
                );
            }
            Err(err) => log::error!("{err:?}"),
        }
    }
}

#[embassy_executor::task]
pub async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        
        if !matches!(controller.is_ap_enabled(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.into(),
                password: PASSWORD.into(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("Starting wifi");
            controller.start().await.unwrap();
            println!("Wifi started!");
        }
        println!("About to connect...");

        match controller.connect().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
pub async fn net_task(stack: &'static Stack<WifiDevice<'static,WifiStaDevice >>) {
    stack.run().await
}