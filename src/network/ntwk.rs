
use embassy_executor::{task as async_task};
use embassy_net::{Stack};
use embassy_time::{Duration, Timer};
use esp_wifi::wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice, WifiState};
use esp_println::println;


#[async_task]
pub async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.get_capabilities());

    const WIFI_CREDS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/network.conf"));

    let (ssid, password) = match WIFI_CREDS.split_once(";") {
        Some((s, p)) => (s.trim(), if p.trim().is_empty() { "" } else { p.trim() }),
        None => {
            println!("Failed to parse WiFi credentials");
            return; }
    };

    println!("Connecting to: {}", ssid);

    let max_retries = 5;
    let mut retries = 0;

    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: ssid.try_into().unwrap(),
                password:password.try_into().unwrap(),
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
            Err(_) => {
                retries += 1;
                println!("Failed to connect to wifi, attempt {}/{}", retries, max_retries);
                if retries >= max_retries {
                    return;
                }
                Timer::after(Duration::from_millis(5000)).await;
            }
        }
    }
}

#[async_task]
pub async fn net_task(stack: &'static Stack<WifiDevice<'static,WifiStaDevice >>) {
    stack.run().await
}

//To-Do convert to generic implementation
/*
#[derive(Debug)]
pub enum WifiError {
    InvalidCredentials,
    ConnectionFailed,
    ControllerError,
    WifiStartError,
    WifiStateError,
}

impl fmt::Display for WifiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WifiError::InvalidCredentials => write!(f, "Failed to parse WiFi credentials"),
            WifiError::ConnectionFailed => write!(f, "Failed to connect to WiFi"),
            WifiError::ControllerError => write!(f, "Controller error occurred"),
            WifiError::WifiStartError => write!(f, "Failed to start WiFi"),
            WifiError::WifiStateError => write!(f, "Failed to get WiFi state"),
        }
    }
}

impl core::fmt::Debug for WifiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait WifiController {
    fn get_capabilities(&self) -> &str;

    fn is_started(&self) -> Result<bool, WifiError>;

    fn set_configuration(&self, config: &Configuration) -> Result<(), WifiError>;

    fn start(&self) -> impl Future<Output = Result<(), WifiError>>;

    fn connect(&self) -> impl Future<Output = Result<(), WifiError>>;

    fn wait_for_event(&self, event: WifiEvent) -> impl Future<Output = ()>;
}

pub trait NetworkStack {
    fn run(&self) -> impl Future<Output = Result<(), WifiError>>;
}

pub trait WifiDriver {
    type Error: fmt::Debug;

    async fn initialize(&self) -> Result<(), Self::Error>;

    async fn connect(&self, controller: impl WifiController) -> Result<(), Self::Error>;

    async fn net_task(&self, stack: impl NetworkStack) -> Result<(), Self::Error>;
}

impl<N: WifiController, S: NetworkStack> WifiDriver for (N, S) {
    type Error = WifiError;

    async fn initialize(&self) -> Result<(), Self::Error> {
        todo!()
    }

    async fn connect(&self, controller: N) -> Result<(), Self::Error> {
        println!("start connection task");
        println!("Device capabilities: {:?}", controller.get_capabilities());

        const WIFI_CREDS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/network.conf"));

        let (ssid, password) = match WIFI_CREDS.split_once(":") {
            Some((s, p)) => (s.trim(), if p.trim().is_empty() { "" } else { p.trim() }),
            None => {
                return Err(WifiError::InvalidCredentials);
            }
        };

        println!("Connecting to: {}", ssid);

        let max_retries = 5;
        let mut retries = 0;

        loop {
            match esp_wifi::wifi::get_wifi_state() {
                WifiState::StaConnected => {
                    // wait until we're no longer connected
                    controller.wait_for_event(WifiEvent::StaDisconnected).await;
                    Timer::after(Duration::from_millis(5000)).await;
                }
                _ => {}
            }

            if !matches!(controller.is_started(), Ok(true)) {
                let client_config = Configuration::Client(ClientConfiguration {
                    ssid: ssid.try_into().unwrap(),
                    password: password.try_into().unwrap(),
                    ..Default::default()
                });
                controller.set_configuration(&client_config).unwrap();
                println!("Starting wifi");
                controller.start().await.unwrap();
                println!("Wifi started!");
            }
            println!("About to connect...");

            match controller.connect().await {
                Ok(_) => {
                    println!("Wifi connected!");
                    return Ok(());
                }
                Err(_) => {
                    retries += 1;
                    println!("Failed to connect to wifi, attempt {}/{}", retries, max_retries);
                    if retries >= max_retries {
                        return Err(WifiError::ConnectionFailed);
                    }
                    Timer::after(Duration::from_millis(5000)).await;
                }
            }
        }
    }

    async fn net_task(&self, stack: S) -> Result<(), Self::Error> {
        stack.run().await.map_err(|_| WifiError::WifiStateError)
    }
}
*/