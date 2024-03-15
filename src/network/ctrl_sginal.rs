use crate::prelude::*;

pub struct ServoCommand {
    pub dx: i32,
    pub dy: i32,
    pub at_rest: bool,
}
pub enum MotorCommand {
    On,
    Off,
    Launch,
}
pub static SERVO_CTRL_SIGNAL: Signal<CriticalSectionRawMutex, ServoCommand> = Signal::new();
pub static MOTOR_CTRL_SIGNAL: Signal<CriticalSectionRawMutex, MotorCommand> = Signal::new();
