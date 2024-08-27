# rusty-robot

Rusty Robot is a modular, scalable, and real-time robotic control system built using Rust. 
The project leverages modern Rust features, including asynchronous programming and a layered architecture, to control a variety of hardware components over wireless networks.

> [!NOTE]
>
> This project is still in the relatively early stages of development, and as such there should be no expectation of API stability.

## Acknowledgement 

- Special thanks to [**Mark S.**](the@wondersmith.dev) the one and only **WonderSmith**
- Thank you to the hardworking teams of [Rust Embedded](https://github.com/rust-embedded), [Embassy-Rs](https://github.com/embassy-rs), [ESP-Rs](https://github.com/esp-rs/esp-hal/blob/main/README.md), and [PicoServe](https://github.com/sammhicks/picoserve) for making it possible to develop this platform
- Thank you to the Rust community for their invaluable resources and support
- Inspired by ROS, YARP and Orca to make a robotics platform for embedded devices that require low resources

## Architecture

              +--------------------------------------------------------+                                                         
              |                        API                             |                                                         
              +--------------------------------------------------------+                                                         
              +--------------------------------------------------------+ +-------+                                               
              |                    Application                         | |       |                                               
              +--------------------------------------------------------+ |       |                                               
              +--------------------------------------------------------+ |       |                                               
              |                   Communications                       | |       |                                               
              +----------------+------------------+--------------------+ |       |                                               
              |Network Services|  Message Manager |Data Synchronization| |       |                                               
              +----------------+------------------+--------------------+ |       |                                               
              +--------------------------------------------------------+ |Tracing|                                               
              |                   Hardware Layer                       | |       |                                               
              |                +-----------------+                     | |       |                                               
              |                |    Middleware   |                     | |       |                                               
              |                +-----------------+                     | |       |                                               
              +--------------------------------------------------------+ |       |                                               
              |          Configurations & Initializations              | |       |                                               
              +---------+     +--------+   +-------+   +---------------+ |       |                                               
              |  ESP32  |     | RP2040 |   | Local |   | Future Boards | |       |                                               
              +---------+-----+--------+---+-------+---+---------------+ +-------+                                               
              +--------------------------------------------------------+                                                         
              |                GPIOs, Sensors, Actuators               |                                                         
              +--------------------------------------------------------+ 

## Roadmap

- Application Layer (integrating hardware and communications)
  * [ ] Create first integrated application example
<br />
<br />

- Communications Layer
    * [ ] Network Services
    * [ ] Message Management
    * [ ] Data synchronization
<br />
<br />
  
- Hardware Layer (supporting ESP32, RP2040, and local testing)
    * [X] ESP32 (Xtensa only)
    * [ ] RP2040
    * [X] Local Testing (Ubuntu Only)


## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution notice

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
