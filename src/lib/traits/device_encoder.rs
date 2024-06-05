use std::fmt::Debug;
use serial_thread::SerialMessage;
use crate::modbus::ModbusId;

pub trait DeviceEncoder<DeviceRequest, DeviceResponse>: Debug + Send
{
    fn request_to_serial(&self, request: DeviceRequest) -> Option<SerialMessage>;
    fn serial_to_response(&self, msg: SerialMessage, request: DeviceRequest, id: ModbusId) -> DeviceResponse;

    fn filter_response(&self, msg: SerialMessage) -> Option<SerialMessage> {
        // filtering: we handle only receive/no response, drop other messages
        match &msg {
            SerialMessage::Receive(data) => {
                
                if data.len() > 6 {Some(msg)} else {
                    log::error!("Receive incomplete response: {:?}", data);
                    Some(SerialMessage::NoResponse)
                }
            },
            SerialMessage::NoResponse => Some(msg),
            _ => None,
        }
    }
}

