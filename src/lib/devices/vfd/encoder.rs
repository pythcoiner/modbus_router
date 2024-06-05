use modbus_core::{Request, Response};
use modbus_core::codec::Encode;
use modbus_core::rtu::crc16;
use serial_thread::SerialMessage;
use crate::devices::vfd::requests::{VfdRequest, VfdResponse, VfdStatus};
use crate::modbus::ModbusId;
use crate::traits::device_encoder::DeviceEncoder;

#[derive(Debug, Clone, Copy)]
pub struct VfdCommands {
    pub cmd_address: u16,
    pub ref_address: u16,
    pub status_address: u16,
    pub fw_value: u16,
    pub rv_value: u16,
    pub stop_value: u16,
}

pub const FRECON: VfdCommands = VfdCommands {
    cmd_address: 0x2000,
    ref_address: 0x2001,
    status_address: 0x3000,
    fw_value: 0x0001,
    rv_value: 0x0002,
    stop_value: 0x0005,
};

pub const MEGMEET: VfdCommands = VfdCommands {
    cmd_address: 0x6400,
    ref_address: 0x6401,
    status_address: 0x6505,
    fw_value: 0x0034,
    rv_value: 0x003c,
    stop_value: 0x0035,
};

#[derive(Debug, Clone, Copy)]
pub struct VfdEncoder {
    commands: VfdCommands,
}

impl VfdEncoder {
    pub fn new(commands: VfdCommands) -> Self {
        VfdEncoder {
            commands,
        }
    }

    /// Decodes a raw Modbus response message into a `VfdResponse`.
    ///
    /// This function takes a raw Modbus message and a `VfdRequest`, decodes the message,
    /// and maps it to an appropriate `VfdResponse`.
    ///
    /// Parameters:
    /// - `msg`: The raw byte array of the Modbus response message.
    /// - `request`: The `VfdRequest` corresponding to the response.
    ///
    /// Returns an `Option<VfdResponse>` which is `Some` with the decoded response if successful,
    /// or `None` if the response cannot be decoded or is not valid for the request.
    fn decode_response(&self, msg: Vec<u8>, request: VfdRequest, vfd: VfdCommands) -> Option<VfdResponse> {
        fn u16_to_i16(u: u16) -> i16 {
            let mut v = (u & 0x7fff) as i16;
            if ((u & 0x8000) >> 8) == 1 {
                v = -v;
            }
            v
        }
        // modbus id is dropped
        let raw_response = &msg[1..];

        log::debug!("VfdEncoder.decode_response({:?}) ", raw_response);
        if let Ok(response) = modbus_core::Response::try_from(raw_response) {
            match (request, response) {
                (VfdRequest::Status(s), Response::ReadHoldingRegisters(data)) => {
                    if data.len() == 1 {
                        let reference = u16_to_i16(data.get(0).expect("at least one word"));
                        if reference == 0 {
                            Some(VfdResponse::Status(VfdStatus::Stop))
                        } else {
                            Some(VfdResponse::Status(VfdStatus::Run(reference)))
                        }
                    } else {
                        log::debug!("VfdEncoder.decode_response() status not match: {:?} / {:?}", s, response);
                        None
                    }
                }
                (VfdRequest::Cmd(_, dir), Response::WriteSingleRegister(addr, value)) => {
                    if addr == vfd.cmd_address && value == dir.into_u16(vfd) {
                        Some(VfdResponse::OK(request))
                    } else {
                        None
                    }
                }
                (VfdRequest::Ref(_, reference), Response::WriteSingleRegister(addr, value)) => {
                    if addr == vfd.ref_address && value == reference {
                        Some(VfdResponse::OK(request))
                    } else {
                        None
                    }
                }
                (VfdRequest::Stop(_), Response::WriteSingleRegister(addr, value)) => {
                    if addr == vfd.cmd_address && value == vfd.stop_value {
                        Some(VfdResponse::OK(request))
                    } else {
                        None
                    }
                }
                (a, b) => {
                    log::debug!("VfdEncoder.decode_response() unrecognized pattern! {:?} / {:?}", a, b);
                    None
                },
            }
        } else {
            log::debug!("VfdEncoder.decode_response() fail to decode response!");
            None
        }
    }
}

impl DeviceEncoder<VfdRequest, VfdResponse> for VfdEncoder {
    fn request_to_serial(&self, request: VfdRequest) -> Option<SerialMessage> {
        let vfd = self.commands;
        let (id, request) = match request {
            VfdRequest::Cmd(id, dir) => (id, Request::WriteSingleRegister(
                vfd.cmd_address, dir.into_u16(vfd))),
            VfdRequest::Ref(id, reference) => {
                (id, Request::WriteSingleRegister(vfd.ref_address, reference))
            }
            VfdRequest::Stop(id) => (id, Request::WriteSingleRegister(vfd.cmd_address, vfd.stop_value)),
            VfdRequest::Status(id) => (id, Request::ReadHoldingRegisters(vfd.status_address, 1)),
        };
        let mut frame: Vec<u8> = vec![id.into()];
        let bytes = &mut [0; 5];
        request.encode(bytes).expect("fixed frame size");
        frame.append(&mut bytes.to_vec());
        let crc = crc16(&frame.to_vec());
        frame.append(&mut vec![((crc & 0xff00) >> 8) as u8, (crc & 0x00ff) as u8]);

        Some(SerialMessage::Send(frame))
    }

    fn serial_to_response(&self, msg: SerialMessage, request: VfdRequest, id: ModbusId) -> VfdResponse {
        log::debug!("VfdEncoder.serial_to_response({:?})", msg);
        match msg.clone() {
            SerialMessage::Receive(data) => {
                if data[0] != id.into() {
                    log::error!("VfdEncoder.serial_to_response() id not match! ({} vs {})", &data[0], {let i: u8 = id.into(); i});
                    VfdResponse::Fail(request)
                } else if let Some(response) = self.decode_response(data, request, self.commands) {
                    response
                } else {
                    log::error!("VfdEncoder.serial_to_response({:?}) fail decoding response!", msg);
                    VfdResponse::Fail(request)
                }
            }
            SerialMessage::NoResponse => {
                log::error!("VfdEncoder.serial_to_response() no response to {:?}!", request);
                VfdResponse::Fail(request)
            }
            _ => { panic!("We should have drop this message in DeviceEncoder::filter_response()")}
        }
    }

}