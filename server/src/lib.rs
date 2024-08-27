//! ## Server Library
//!
//! This library provides the implementation for a WebSocket server and
//! message handling system, designed to operate in a `no_std` environment
//! when compiled with specific features and target configurations.
//! It includes modules for server functionality and message processing.

#![allow(unexpected_cfgs, unused_qualifications)]
#![no_std]

#![cfg_attr(
    all(feature = "mcu", target_os = "none", target_vendor = "unknown"),
    no_std
)]

/// Server Module
///
/// This module contains the implementation of the WebSocket server, including
/// server configuration and WebSocket handling. It provides functionality to
/// start the server, manage WebSocket connections, and process incoming messages.
pub mod server;

/// Messages Module
///
/// This module defines the message passing mechanism for the WebSocket server.
/// It includes message definitions, a global channel for message passing, and
/// a task for routing commands to appropriate handlers based on the message type.
pub mod messages;
