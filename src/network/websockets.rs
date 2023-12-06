//! Web Sockets for Using picoserve

#[allow(unused_imports)]
extern crate alloc;

use crate::prelude::*;

use picoserve::{response::{status, ws::Message, WebSocketUpgrade}, ResponseSent, extract::FromRequest};


struct NewMessageRejection(core::str::Utf8Error);

impl picoserve::response::IntoResponse for NewMessageRejection {
    async fn write_to<W: picoserve::response::ResponseWriter>(
        self,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        (
            status::BAD_REQUEST,
            format_args!("Body is not UTF-8: {}\n", self.0),
        )
            .write_to(response_writer)
            .await
    }
}

struct NewMessage(alloc::string::String);

impl<State> picoserve::extract::FromRequest<State> for NewMessage {
    type Rejection = NewMessageRejection;

    async fn from_request(
        _state: &State,
        request: &picoserve::request::Request<'_>,
    ) -> Result<Self, Self::Rejection> {
        core::str::from_utf8(request.body())
            .map(|message| NewMessage(message.into()))
            .map_err(NewMessageRejection)
    }
}

pub struct WebsocketHandler {}

impl picoserve::response::ws::WebSocketCallback for WebsocketHandler {
    async fn run<R: picoserve::io::Read, W: picoserve::io::Write<Error = R::Error>>(
        self,
        mut rx: picoserve::response::ws::SocketRx<R>,
        mut tx: picoserve::response::ws::SocketTx<W>,
    ) -> Result<(), W::Error> {
        

        let mut message_buffer = [0; 1024];

        loop {
            match rx.next_message(&mut message_buffer).await {
                Ok(message) => match message {
                    Message::Text(text) => {
                        // Handle text message
                        println!("Received text message: {}", text);
                        handle_message(text, &mut tx).await?;
                    }
                    Message::Binary(data) => {
                        // Handle binary message
                        println!("Received binary message with {} bytes", data.len());
                        //tx.send_binary(data).await?;
                    }
                    _ => (),
                },
                Err(e) => {
                    // Handle error
                    println!("Error while reading message: {:?}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}

async fn handle_message<W: picoserve::io::Write>(
    message: &str, 
    tx: &mut picoserve::response::ws::SocketTx<W>
) -> Result<(), W::Error> {
    if message.starts_with("auth:") {
        println!("Authentication message: {}", message);
        tx.send_text("Acknowledged auth message").await?;
    } else if message.starts_with("dc:") {
        println!("DC motor message: {}", message);
        tx.send_text("Acknowledged DC motor message").await?;
    /*     if message == "dc:on" {
            // Turn on DC motor
        } else if message == "dc:off" {
            // Turn off DC motor
        } else if message == "dc:launch" {
            // Handle launch command
        } */
    } else if message.starts_with("servo:") {
        // Handle servo messages
        let parts: alloc::vec::Vec<&str> = message.split(':').collect();
        if let (Some(dx_str), Some(dy_str)) = (parts.get(1), parts.get(2)) {
            if let (Ok(dx), Ok(dy)) = (dx_str.parse::<i32>(), dy_str.parse::<i32>()) {
                // Use dx and dy to control the servo
                println!("Servo message: dx = {}, dy = {}", dx, dy);
                //tx.send_text(&format!("Received servo positions: dx = {}, dy = {}", dx, dy)).await?;
            }
        }
    }

    Ok(())
}
struct State;

// WebSocket route handler function
pub async fn websocket_handler(request: &picoserve::request::Request<'_>) -> impl picoserve::response::IntoResponse {
    println!("WebSocket upgrade request received");
    let websocket_upgrade = WebSocketUpgrade::from_request(&State, request).await;
    match websocket_upgrade {
        Ok(upgrade) => {
            println!("WebSocket upgrade successful");
            upgrade.on_upgrade(WebsocketHandler {})
        },
        Err(_rejection) => {
            println!("WebSocket upgrade failed");
            todo!()
        },
    }
}
