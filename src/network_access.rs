//! ## TKR's `Network Access` Module
//!
//! This "module" contains embassy async tasks for connecting
//! to both STA and AP at the same time. The intent is for the 
//! end user to be able to access the ESP32 front-end either by
//! perferable connecting to the local wifi or by AP.


#[allow(unused_imports)]

use crate::prelude::*;


const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[embassy_executor::task]
pub async fn new_network_service(spawner: Spawner, wifi_ap_interface: WifiDevice<'static, WifiApDevice>, wifi_sta_interface: WifiDevice<'static, WifiStaDevice>, mut wifi_controller: WifiController<'static>){

    let ap_config = Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 2, 1), 24),
        gateway: Some(Ipv4Address::from_bytes(&[192, 168, 2, 1])),
        dns_servers: Default::default(),
    });

    let sta_config = Config::dhcpv4(Default::default());

    let seed = 1234; // Should be updated to use a secure random seed

    // Init network stacks
    let ap_stack = &*make_static!(Stack::new(
        wifi_ap_interface,
        ap_config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    let sta_stack = &*make_static!(Stack::new(
        wifi_sta_interface,
        sta_config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    let client_config = Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASSWORD.into(),
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "esp-wifi".into(),
            ..Default::default()
        },
    );
    wifi_controller.set_configuration(&client_config).unwrap();

    spawner.spawn(connection(wifi_controller)).ok();
    spawner.spawn(ap_task(&ap_stack)).ok();
    spawner.spawn(sta_task(&sta_stack)).ok();

    loop {
        if sta_stack.is_link_up() {
            break;
        }
        println!("Waiting for IP...");
        Timer::after(Duration::from_millis(500)).await;
    }
    loop {
        if ap_stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    println!("Connect to the AP `esp-wifi` and point your browser to http://192.168.2.1:8080/");
    println!("Use a static IP in the range 192.168.2.2 .. 192.168.2.255, use gateway 192.168.2.1");

    let mut ap_rx_buffer = [0; 1536];
    let mut ap_tx_buffer = [0; 1536];

    let mut ap_socket = TcpSocket::new(&ap_stack, &mut ap_rx_buffer, &mut ap_tx_buffer);
    ap_socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

    let mut sta_rx_buffer = [0; 1536];
    let mut sta_tx_buffer = [0; 1536];

    let mut sta_socket = TcpSocket::new(&sta_stack, &mut sta_rx_buffer, &mut sta_tx_buffer);
    sta_socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

    loop {
        println!("Wait for connection...");
        let r = ap_socket
            .accept(IpListenEndpoint {
                addr: None,
                port: 8080,
            })
            .await;
        println!("Connected...");

        if let Err(e) = r {
            println!("connect error: {:?}", e);
            continue;
        }

        use embedded_io_async::Write;

        let mut buffer = [0u8; 1024];
        let mut pos = 0;
        loop {
            match ap_socket.read(&mut buffer).await {
                Ok(0) => {
                    println!("AP read EOF");
                    break;
                }
                Ok(len) => {
                    let to_print =
                        unsafe { core::str::from_utf8_unchecked(&buffer[..(pos + len)]) };

                    if to_print.contains("\r\n\r\n") {
                        print!("{}", to_print);
                        println!();
                        break;
                    }

                    pos += len;
                }
                Err(e) => {
                    println!("AP read error: {:?}", e);
                    break;
                }
            };
        }

        if sta_stack.is_link_up() {
            let remote_endpoint = (Ipv4Address::new(142, 250, 185, 115), 80);
            println!("connecting...");
            let r = sta_socket.connect(remote_endpoint).await;
            if let Err(e) = r {
                println!("STA connect error: {:?}", e);
                continue;
            }

            //use embedded_io_async::Write;
            let r = sta_socket
                .write_all(b"GET / HTTP/1.0\r\nHost: www.mobile-j.de\r\n\r\n")
                .await;

            if let Err(e) = r {
                println!("STA write error: {:?}", e);

                let r = ap_socket
                    .write_all(
                        b"HTTP/1.0 500 Internal Server Error\r\n\r\n\
                        <html>\
                            <body>\
                                <h1>Hello Rust! Hello esp-wifi! STA failed to send request.</h1>\
                            </body>\
                        </html>\r\n\
                        ",
                    )
                    .await;
                if let Err(e) = r {
                    println!("AP write error: {:?}", e);
                }
            } else {
                let r = sta_socket.flush().await;
                if let Err(e) = r {
                    println!("STA flush error: {:?}", e);
                } else {
                    println!("connected!");
                    let mut buf = [0; 1024];
                    loop {
                        match sta_socket.read(&mut buf).await {
                            Ok(0) => {
                                println!("STA read EOF");
                                break;
                            }
                            Ok(n) => {
                                let r = ap_socket.write_all(&buf[..n]).await;
                                if let Err(e) = r {
                                    println!("AP write error: {:?}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                println!("STA read error: {:?}", e);
                                break;
                            }
                        }
                    }
                }
            }

            sta_socket.close();
        } else {
            let r = ap_socket
                .write_all(
                    b"HTTP/1.0 200 OK\r\n\r\n\
                    <html>\
                        <body>\
                            <h1>Hello Rust! Hello esp-wifi! STA is not connected.</h1>\
                        </body>\
                    </html>\r\n\
                    ",
                )
                .await;
            if let Err(e) = r {
                println!("AP write error: {:?}", e);
            }
        }

        let r = ap_socket.flush().await;
        if let Err(e) = r {
            println!("AP flush error: {:?}", e);
        }
        Timer::after(Duration::from_millis(1000)).await;

        ap_socket.close();
        Timer::after(Duration::from_millis(1000)).await;

        ap_socket.abort();
    }
}    

#[embassy_executor::task]
async fn connection(mut wifi_controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", wifi_controller.get_capabilities());

    println!("Starting wifi");
    wifi_controller.start().await.unwrap();
    println!("Wifi started!");

    loop {
        match esp_wifi::wifi::get_ap_state() {
            WifiState::ApStarted => {
                println!("About to connect...");

                match wifi_controller.connect().await {
                    Ok(_) => {
                        // wait until we're no longer connected
                        wifi_controller.wait_for_event(WifiEvent::StaDisconnected).await;
                        println!("STA disconnected");
                    }
                    Err(e) => {
                        println!("Failed to connect to wifi: {e:?}");
                        Timer::after(Duration::from_millis(5000)).await;
                    }
                }
            }
            _ => return
        }
    }
}



#[embassy_executor::task]
async fn ap_task(stack: &'static Stack<WifiDevice<'static, WifiApDevice>>) {
    stack.run().await
}

#[embassy_executor::task]
async fn sta_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
} 


