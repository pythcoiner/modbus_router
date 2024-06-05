use crate::devices::vfd::encoder::VfdCommands;
use crate::devices::vfd::requests::Dir::Fw;
use crate::modbus::ModbusId;


#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
/// Specifies the direction for a command.
///
/// ## Variants
/// - `Fw`: Forward direction.
/// - `Rv`: Reverse direction.
pub enum Dir {
    Fw,
    Rv,
}

#[allow(clippy::from_over_into)]
impl Dir {
    pub fn into_u16(self, vfd: VfdCommands) -> u16 {
        if self == Fw {
            vfd.fw_value
        } else {
            vfd.rv_value
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
/// Represents a request to be sent to the Vfd Poller.
///
/// ## Variants
/// - `Cmd(Dir)`: Command with a direction (`Fw` or `Rv`).
/// - `Ref(u16)`: A reference value.
/// - `Stop`: Command to stop the VFD.
/// - `Status`: Request for the current status of the VFD.
pub enum VfdRequest {
    Cmd(ModbusId, Dir),
    Ref(ModbusId, u16),
    Stop(ModbusId),
    Status(ModbusId),
}


#[allow(unused)]
#[derive(Debug, Clone, Copy)]
/// Represents a response from the Vfd Poller.
///
/// ## Variants
/// - `OK(VfdRequest)`: Successful acknowledgment of a `VfdRequest`.
/// - `Fail(VfdRequest)`: Indicates a failure in processing a `VfdRequest`.
/// - `Status(VfdStatus)`: Provides the status of the VFD.
/// - `Poll`: Indicates a polling request in order to VfdAxis send a Batch to VfdPoller.
pub enum VfdResponse {
    OK(VfdRequest),
    Fail(VfdRequest),
    Status(VfdStatus),
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
/// Indicates the current status of a Vfd.
///
/// ## Variants
/// - `Run(i16)`: Indicates the VFD is running with a specific value (speed or power level).
/// - `Stop`: Indicates the VFD is stopped.
/// - `None`: Used for uninitialized or default status.
pub enum VfdStatus {
    Run(i16),
    Stop,
    None,
}