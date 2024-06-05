use crate::devices::joystick::device::JoystickType;
use crate::modbus::ModbusId;

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum JoystickRequest {
    Status(ModbusId, JoystickType),
}


#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum JoystickResponse {
    Fail(JoystickRequest),
    Status(JoystickStatus),
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoystickStatus {
    Joystick([u16;4]),
    JoystickWithThumb([u16;5]),
    None,
}