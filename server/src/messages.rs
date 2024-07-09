//! ## Messages Module
//!
//! This module handles the message passing mechanism for the WebSocket server.
//! It defines the structure of the messages and provides a channel for sending
//! and receiving commands to control hardware components such as motors and servos.

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use hardware::{MotorCommand, ServoCommand};

/// Global Channel for WebSocket Messages
///
/// This static channel is used to send and receive `WebSocketMessage` instances.
/// It employs a `CriticalSectionRawMutex` for synchronization and has a capacity of 64 messages.
pub static CHANNEL: Channel<CriticalSectionRawMutex, WebSocketMessage, 64> = Channel::new();

/// WebSocket Message Enum
///
/// This enum defines the different types of messages that can be received via WebSocket.
/// It includes commands for motors, servos, and combined motor and servo commands.
///
/// # Variants
///
/// - `Motor(MotorCommand)`: A command to control a motor.
/// - `Servo(ServoCommand)`: A command to control a servo.
/// - `MotorAndServo { motor, servo }`: A command that includes both motor and servo commands.
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

/// Command Router Task
///
/// This asynchronous task continuously listens for incoming `WebSocketMessage` instances from the `CHANNEL`.
/// It routes the messages to the appropriate handlers based on their type.
#[embassy_executor::task]
pub async fn command_router() {
    loop {
        match CHANNEL.receiver().receive().await {
            WebSocketMessage::Motor(command) => {
                tracing::info!("Received Motor Command: {:?}", command);
            }
            WebSocketMessage::Servo(command) => {
                tracing::info!("Received Servo Command: {:?}", command);
            }
            WebSocketMessage::MotorAndServo { motor, servo } => {
                tracing::info!(
                    "Received Motor Command: {:?} and Servo Command: {:?}",
                    motor,
                    servo
                );
            }
        }
    }
}
