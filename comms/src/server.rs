//! ## Server Module
//!
//! This module contains the implementation of the comms, including
//! the WebSocket handling and comms configuration. It uses the
//! `picoserve` and `embassy` crates to manage network operations
//! and timing.

use embassy_net::{driver::Driver as NetworkDriver, Stack};
use embassy_time::Duration;
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

use crate::messages;

/// Runs the comms with the given configuration.
///
/// This function initializes the comms and starts listening for
/// incoming connections. It sets up the necessary routing and
/// handles WebSocket connections.
///
/// # Parameters
///
/// - `id`: The identifier for the comms instance.
/// - `port`: The port on which the comms will listen.
/// - `stack`: A reference to the network stack.
/// - `config`: An optional configuration for the comms.
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

/// Timer implementation for the comms.
///
/// This struct provides a timer that integrates with the `embassy_time`
/// crate, allowing for asynchronous operations with timeout functionality.
pub struct ServerTimer;

#[allow(unused_qualifications)]
impl picoserve::Timer for ServerTimer
{
    type Duration = embassy_time::Duration;
    type TimeoutError = embassy_time::TimeoutError;

    /// Runs a future with a timeout.
    ///
    /// This method wraps the provided future with a timeout, returning
    /// an error if the future does not complete within the specified duration.
    ///
    /// # Parameters
    ///
    /// - `duration`: The timeout duration.
    /// - `future`: The future to be run with the timeout.
    ///
    /// # Returns
    ///
    /// A result containing either the future's output or a timeout error.
    async fn run_with_timeout<F: core::future::Future>(
        &mut self,
        duration: Self::Duration,
        future: F,
    ) -> Result<F::Output, Self::TimeoutError>
    {
        embassy_time::with_timeout(duration, future).await
    }
}

/// WebSocket implementation.
///
/// This struct handles WebSocket connections, processing incoming messages
/// and responding appropriately. It uses the `picoserve` crate's WebSocket
/// callback mechanism to handle messages.
pub struct WebSocket;

impl WebSocketCallback for WebSocket
{
    /// Runs the WebSocket connection, processing incoming messages.
    ///
    /// This method is called when a WebSocket connection is established.
    /// It reads messages from the client and processes them, sending
    /// responses as needed.
    ///
    /// # Parameters
    ///
    /// - `rx`: The socket for receiving messages.
    /// - `tx`: The socket for sending messages.
    ///
    /// # Returns
    ///
    /// A result indicating whether the connection was handled successfully or
    /// if there was an error.
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
                Ok(Message::Text(data)) => {
                    match serde_json::from_str::<messages::WebSocketMessage>(data) {
                        Ok(message) => {
                            messages::CHANNEL.send(message).await;
                            tx.send_text(data).await?
                        }
                        Err(error) => {
                            tracing::error!(?error, "error deserializing incoming message")
                        }
                    }
                }
                Ok(Message::Binary(data)) => match serde_json::from_slice::<
                    messages::WebSocketMessage,
                >(data)
                {
                    Ok(message) => {
                        messages::CHANNEL.send(message).await;
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
