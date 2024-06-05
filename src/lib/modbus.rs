
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents the type of frame in a PLC communication.
///
/// ## Variants
/// - `Request`: A frame representing a request.
/// - `Response`: A frame representing a response.
/// - `None`: Represents the absence of a frame or an uninitialized state.
pub enum FrameType {
    Request,
    Response,
    None,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Defines the function or command being communicated in a PLC request/response.
///
/// ## Variants
/// - `Run`: Represents a command to run or execute an operation.
/// - `Stop`: Represents a command to stop an operation.
/// - `Status`: Represents a request or response pertaining to the status.
/// - `None`: Indicates no specific function, used for uninitialized or default states.
pub enum FunctionType {
    Run,
    Stop,
    Status,
    None,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
/// Represents the identifier for a Modbus device or entity.
///
/// ## Variants
/// - `Id(u8)`: A specific ID for a Modbus device (1-247).
/// - `Broadcast`: Used for broadcasting to all devices (0).
/// - `Reserved`: Reserved IDs (248-255).
pub enum ModbusId {
    Id(u8),
    Broadcast,
    Reserved,
}

impl From<u8> for ModbusId {
    fn from(value: u8) -> Self {
        if value == 0 {
            ModbusId::Broadcast
        } else if value < 248 {
            ModbusId::Id(value)
        } else {
            ModbusId::Reserved
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<u8> for ModbusId {
    fn into(self) -> u8 {
        match self {
            ModbusId::Id(id) => id,
            ModbusId::Broadcast => 0,
            ModbusId::Reserved => 255,
        }
    }
}