use modbus_core::codec::Encode;
use modbus_core::{Request, Response};
use modbus_core::rtu::crc16;
use serial_thread::SerialMessage;
use crate::devices::joystick::device::JoystickType;
use crate::devices::joystick::requests::{JoystickRequest, JoystickResponse, JoystickStatus};
use crate::modbus::ModbusId;
use crate::traits::device_encoder::DeviceEncoder;

#[derive(Debug, Clone, Copy)]
pub struct JoystickEncoder {
    joystick_type: JoystickType,
}

impl JoystickEncoder {
    pub fn new(joystick_type: JoystickType) -> Self {
        JoystickEncoder {
            joystick_type,
        }
    }
    
    fn decode_response(&self, data: Vec<u8>) -> Option<JoystickStatus>{
        // drop id
        let raw_response = &data[1..];
        if let Ok(Response::ReadHoldingRegisters(payload)) = modbus_core::Response::try_from(raw_response) {
            let u16_data: Vec<u16> = payload.into_iter().collect();
            match (self.joystick_type, u16_data) {
                (JoystickType::Joystick, d) => {
                    if d.len() == 4 {
                        let p: Result<[u16;4], _> = d[..].try_into();
                        if let Ok(status) = p {
                            return Some(JoystickStatus::Joystick(status));
                        }
                    }
                },
                (JoystickType::JoystickWithThumb, d) => {
                    if d.len() == 5 {
                        let p: Result<[u16;5], _> = d[..].try_into();
                        if let Ok(status) = p {
                            return Some(JoystickStatus::JoystickWithThumb(status));
                        }
                    }
                }
            }
        }
        None
    }
    
}

impl DeviceEncoder<JoystickRequest, JoystickResponse> for JoystickEncoder {
    fn request_to_serial(&self, request: JoystickRequest) -> Option<SerialMessage> {
        let (id, request) = match &request { 
            JoystickRequest::Status(id, JoystickType::Joystick) => {
                (id, Request::ReadHoldingRegisters(0x4001u16, 4u16))
            },
            JoystickRequest::Status(id, JoystickType::JoystickWithThumb) => {
                (id, Request::ReadHoldingRegisters(0x4001u16, 5u16))
            }
        };

        let mut frame: Vec<u8> = vec![(*id).into()];
        let bytes = &mut [0; 6];
        request.encode(bytes).expect("fixed frame size");
        frame.append(&mut bytes.to_vec());
        let crc = crc16(&frame.to_vec());
        frame.append(&mut vec![((crc & 0xff00) >> 8) as u8, (crc & 0x00ff) as u8]);

        Some(SerialMessage::Send(frame))
    }

    fn serial_to_response(&self, msg: SerialMessage, request: JoystickRequest, id: ModbusId) -> JoystickResponse {
        match msg.clone() {
            SerialMessage::Receive(data) => {
                if data[0] == id.into() {
                    if let Some(response) = self.decode_response(data) {
                        return JoystickResponse::Status(response);
                    }
                }
                JoystickResponse::Fail(request)
            },
            SerialMessage::NoResponse => {
                JoystickResponse::Fail(request)
            }
            _ => {
                panic!("This message should have been filtered out before{:?}", msg);
            }
        }
    }
}