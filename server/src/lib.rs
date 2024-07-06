#![no_std]
#![feature(type_alias_impl_trait)]

use embassy_net::{driver::Driver as NetworkDriver, Stack};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Duration;
use hardware::{MotorCommand, ServoCommand};
use picoserve::{
    io::embedded_io_async as embedded_aio,
    response::ws::{
        Message,
        ReadMessageError,
        SocketRx,
        SocketTx,
        WebSocketCallback,
        WebSocketUpgrade,
    },
    Router,
};

pub async fn run<Driver: NetworkDriver>(
    id: usize,
    port: u16,
    stack: &'static Stack<Driver>,
    config: Option<&'static picoserve::Config<Duration>>,
) -> !
{
    let default_config = picoserve::Config::new(picoserve::Timeouts {
        start_read_request: Some(Duration::from_secs(5)),
        read_request: Some(Duration::from_secs(1)),
        write: Some(Duration::from_secs(5)),
    });

    let config = config.unwrap_or(&default_config);

    let router = Router::new().route(
        "/ws",
        picoserve::routing::get(|upgrade: WebSocketUpgrade| upgrade.on_upgrade(WebSocket)),
    );

    let (mut rx_buffer, mut tx_buffer, mut http_buffer) = ([0; 1024], [0; 1024], [0; 256]);

    picoserve::listen_and_serve(
        id,
        &router,
        config,
        stack,
        port,
        &mut rx_buffer,
        &mut tx_buffer,
        &mut http_buffer,
    )
    .await
}

pub struct ServerTimer;

#[allow(unused_qualifications)]
impl picoserve::Timer for ServerTimer {
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

static CHANNEL: Channel<CriticalSectionRawMutex, WebSocketMessage, 64> = Channel::new();

pub struct WebSocket;

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum WebSocketMessage {
    Motor(MotorCommand),
    Servo(ServoCommand),
    MotorAndServo {
        motor: MotorCommand,
        servo: ServoCommand,
    },
    // HandlerResponse(String),
}

impl WebSocketCallback for WebSocket {
    async fn run<Reader, Writer>(
        self,
        mut rx: SocketRx<Reader>,
        mut tx: SocketTx<Writer>,
    ) -> Result<(), Writer::Error>
    where
        Reader: embedded_aio::Read,
        Writer: embedded_aio::Write<Error = Reader::Error>,
    {
        let mut buffer = [0; 1024];

        let close_reason = loop {
            match rx.next_message(&mut buffer).await {
                Ok(Message::Pong(_)) => continue,
                Ok(Message::Ping(data)) => tx.send_pong(data).await?,
                Ok(Message::Close(reason)) => {
                    tracing::info!(?reason, "websocket closed");
                    break None;
                }
                Ok(Message::Text(data)) => match serde_json::from_str::<WebSocketMessage>(data) {
                    Ok(message) => {
                        CHANNEL.send(message).await;
                        tx.send_text(data).await?
                    }
                    Err(error) => tracing::error!(?error, "error deserializing incoming message"),
                },
                Ok(Message::Binary(data)) => match serde_json::from_slice::<WebSocketMessage>(data)
                {
                    Ok(message) => {
                        CHANNEL.send(message).await;
                        tx.send_binary(data).await?
                    }
                    Err(error) => tracing::error!(?error, "error deserializing incoming message"),
                },
                Err(error) => {
                    tracing::error!(?error, "websocket error");

                    let code = match error {
                        ReadMessageError::TextIsNotUtf8 => 1007,
                        ReadMessageError::ReservedOpcode(_) => 1003,
                        ReadMessageError::ReadFrameError(_)
                        | ReadMessageError::UnexpectedMessageStart
                        | ReadMessageError::MessageStartsWithContinuation => 1002,
                        ReadMessageError::Io(err) => return Err(err),
                    };

                    break Some((code, "Websocket Error"));
                }
            };
        };

        tx.close(close_reason).await
    }
}
