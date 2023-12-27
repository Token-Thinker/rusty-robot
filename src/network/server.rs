//! ## TKR's `Server` Module
//!
//! This "module" contains embassy async tasks for creating a
//! server using `picoserve` crate. The intent is for the 
//! end user to be able to access the ESP32 front-end by using
//! the ip address obtained and port 8080.


#[allow(unused_imports)]

use crate::prelude::*;
extern crate alloc;
use crate::network::http::*;

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


pub async fn get_site() -> impl IntoResponse {
    (
        [
            ("Content-Type", "text/html; charset=utf-8"),
            ("Content-Encoding", "gzip")
        ],

        INDEX_HTML_GZ
    )
}

#[embassy_executor::task]
pub async fn server(
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    config: &'static picoserve::Config<Duration>,
    //sender: Sender<'static, NoopRawMutex, MoveCommand,QUEUE_SIZE>
    spawner: Spawner
) -> ! {
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

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

    println!("Starting WS Communication...");
    spawner.spawn(servo_server(stack, config)).ok();
    spawner.spawn(motor_server(stack, config)).ok();

    loop {
        let mut socket = embassy_net::tcp::TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

        log::info!("Listening on TCP:80...");
        if let Err(e) = socket.accept(80).await {
            log::warn!("accept error: {:?}", e);
            continue;
        }

        log::info!(
            "Received connection from {:?}",
            socket.remote_endpoint()
        );

        let (socket_rx, socket_tx) = socket.split();

        let http_app = Router::new()
            .route("/", get(get_site));

        match picoserve::serve(
            &http_app,
            EmbassyTimer,
            config,
            &mut [0; 1024],
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
pub async fn servo_server(
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    config: &'static picoserve::Config<Duration>,
) -> ! {
    let mut servo_rx_buffer = [0; 1024];
    let mut servo_tx_buffer = [0; 1024];

    loop {
        let mut servo_socket = embassy_net::tcp::TcpSocket::new(stack, &mut servo_rx_buffer, &mut servo_tx_buffer);

        log::info!("Listening on TCP:81...");
        if let Err(e) = servo_socket.accept(81).await {
            log::warn!("accept error: {:?}", e);
            continue;
        }

        log::info!(
            "Received connection from {:?}",
            servo_socket.remote_endpoint()
        );

        let (servo_socket_rx, servo_socket_tx) = servo_socket.split();

        let servo_app = Router::new()
                .route(
                "/ws",
                get(|upgrade: picoserve::response::ws::WebSocketUpgrade| {
                upgrade.on_upgrade(crate::network::websockets::WebsocketHandler {})
                }),
            );

        
        match picoserve::serve(
            &servo_app,
            EmbassyTimer,
            config,
            &mut [0; 1024],
            servo_socket_rx,
            servo_socket_tx,
        )
        .await
        {
            Ok(handled_requests_count) => {
                log::info!(
                    "{handled_requests_count} requests handled from {:?}",
                    servo_socket.remote_endpoint()
                );
            }
            Err(err) => log::error!("{err:?}"),
        }
    }
}

#[embassy_executor::task]
pub async fn motor_server(
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    config: &'static picoserve::Config<Duration>,
) -> ! {
    let mut motor_rx_buffer = [0; 512];
    let mut motor_tx_buffer = [0; 512];

    loop {
        let mut motor_socket = embassy_net::tcp::TcpSocket::new(stack, &mut motor_rx_buffer, &mut motor_tx_buffer);

        log::info!("Listening on TCP:82...");
        if let Err(e) = motor_socket.accept(82).await {
            log::warn!("accept error: {:?}", e);
            continue;
        }

        log::info!(
            "Received connection from {:?}",
            motor_socket.remote_endpoint()
        );

        let (motor_socket_rx, motor_socket_tx) = motor_socket.split();

        let motor_app = Router::new()
                .route(
                "/ws",
                get(|upgrade: picoserve::response::ws::WebSocketUpgrade| {
                upgrade.on_upgrade(crate::network::websockets::WebsocketHandler {})
                }),
            );

        
        match picoserve::serve(
            &motor_app,
            EmbassyTimer,
            config,
            &mut [0; 512],
            motor_socket_rx,
            motor_socket_tx,
        )
        .await
        {
            Ok(handled_requests_count) => {
                log::info!(
                    "{handled_requests_count} requests handled from {:?}",
                    motor_socket.remote_endpoint()
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