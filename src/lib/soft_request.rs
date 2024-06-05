use modbus_core::rtu::crc16;
use crate::devices::vfd::requests::VfdStatus;
use crate::error::VfdError;
use crate::modbus::{FrameType, FunctionType, ModbusId};
use crate::traits::request::{RequestFn, ResponseFn};


#[allow(unused)]
#[derive(Debug, Clone, Copy)]
/// # `SoftRequest`
///
/// This enum represents the request format for communication from a PLC Controller.
///
/// All frames passed through `stdin` / `stdout` are 8 bytes long and follow the standard format described here.
/// The payload is only 6 bytes, with a CRC16 from the Modbus protocol used for validation, its unlikely to have data
/// malleability on pipes (stdin/stdout) but we can have slippage so we can detect it w/ CRC check.
///
/// ## Format
///  [MODBUS_ID ,TYPE ,FUNCTION_CODE ,DATA1 ,DATA2 ,DATA3 ,CRC ,CRC]
///
/// - `MODBUS_ID`:
///   - `0` -> Broadcast
///   - `1-247` -> Device ID
///   - `248-255` -> Reserved
/// - `TYPE`:
///   - `1` -> Vfd Request 
///   - `2` -> Vfd Response 
///   - `3` -> Joystick Request
///   - `4` -> Joystick response
/// - `Vfd FUNCTION_CODE` and corresponding data layout:
///   - `1` -> Run: DATA1 = SIGN, DATA2 = Reference MSB, DATA3 = Reference LSB (encoded as i16 without sign)
///   - `2` -> Stop: DATA1, DATA2, DATA3 = `0`
///   - `3` -> Status: DATA1, DATA2, DATA3 = `0`
/// - `Joystick FUNCTION_CODE` and corresponding data layout:
///   - `1` -> X Position: DATA1 = SIGN, DATA2 = X Position MSB, DATA3 = X Position LSB (encoded 
///     as u16 without sign) 
///   - `2` -> Y Position: DATA1 = SIGN, DATA2 = Y Position MSB, DATA3 = Y Position LSB (encoded 
///     as u16 without sign) 
///   - `3` -> Button state: DATA1 = Button # , DATA2 = Button state ( `0` = released, `1` = pressed)
///         state mask ( if mask bit is 1 => the relevant button bit should be updated ) DATA3 = `0`
///   - `4` -> X Thumb Position: DATA1 = SIGN, DATA2 = X Thumb Position MSB, DATA3 = X Thumb 
///     Position LSB (encoded as u16 without sign) DATA3 = `0`
///   - `5` -> Y Thumb Position: DATA1 = SIGN, DATA2 = Y Thumb Position MSB, DATA3 = Y Thumb 
///     Position LSB (encoded as u16 without sign) DATA3 = `0`
///
/// ## Variants
/// - `Run`: Contains a `ModbusId` and a reference as `i16`.
/// - `Stop`: Contains a `ModbusId`.
/// - `Status`: Contains a `ModbusId`.
pub enum SoftRequest {
    Run(ModbusId, i16),
    Stop(ModbusId),
    Status(ModbusId),
}

impl RequestFn for SoftRequest {
    fn from(raw: Vec<u8>) -> Option<Box<Self>> {
        match SoftRequest::try_from(raw.as_slice()) {
            Ok(r) => {
                Some(Box::new(r))
            }
            Err(e) => {
                log::error!("Request.from<u8>({:?}) fail: {:?}", raw, e);
                None
            }
        }
        
    }

    fn id(&self) -> ModbusId {
        match self {
            SoftRequest::Run(id, _) | SoftRequest::Stop(id) | SoftRequest::Status(id) => *id,
        }
    }

    fn new_id(&self, id: ModbusId) -> Box<Self> {
        let out = match self {
            SoftRequest::Run(_, r) => SoftRequest::Run(id, *r),
            SoftRequest::Stop(_) => SoftRequest::Stop(id),
            SoftRequest::Status(_) => SoftRequest::Status(id),
        };
        Box::new(out)
    }
}

impl TryFrom<&[u8]> for SoftRequest {
    type Error = VfdError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == 8 {
            let frame: &[u8; 8] = value.try_into().map_err(|_| VfdError::WrongFrameLength)?;

            // check for crc
            let crc = crc16(&frame[..frame.len()-2]);
            let crc0 = ((crc & 0xff00) >> 8) as u8;
            let crc1 = (crc & 0x00ff) as u8;
            let crc = &[crc0, crc1];
            let frame_crc: &[u8; 2] = (&frame[frame.len() - 2..frame.len()])
                .try_into()
                .expect("cannot fail");
            if crc != frame_crc {
                return Err(VfdError::WrongCrc);
            }

            // deserialize
            let id: ModbusId = frame[0].into();

            if id == ModbusId::Reserved {
                return Err(VfdError::WrongModbusId);
            }

            let frame_type = match &frame[1] {
                0x01 => FrameType::Request,
                0x02 => FrameType::Response,
                _ => FrameType::None,
            };
            if (frame_type == FrameType::None) || (frame_type == FrameType::Response) {
                return Err(VfdError::WrongFrameType);
            }

            let fn_type = match &frame[2] {
                1 => FunctionType::Run,
                2 => FunctionType::Stop,
                3 => FunctionType::Status,
                _ => FunctionType::None,
            };

            let mut run_ref = 0i16;
            if fn_type == FunctionType::Run {
                let reference = ((frame[4] as u16) << 8) | (frame[5] as u16);
                if reference > (i16::MAX as u16) {
                    return Err(VfdError::WrongRefValue);
                }
                let mut reference = reference as i16;
                match frame[3] {
                    0 => {}
                    1 => {
                        reference = -reference;
                    }
                    _ => {
                        return Err(VfdError::WrongRefSign);
                    }
                }
                run_ref = reference;
            }

            match fn_type {
                FunctionType::Run => Ok(SoftRequest::Run(id, run_ref)),
                FunctionType::Status => Ok(SoftRequest::Status(id)),
                FunctionType::Stop => Ok(SoftRequest::Stop(id)),
                FunctionType::None => Err(VfdError::WrongFunctionType),
            }
        } else {
            Err(VfdError::WrongFrameLength)
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
/// # `SoftResponse`
///
/// This enum represents the response to the PLC Controller.
///
/// For the format and details of the request structure, see [`SoftRequest`].
///
/// ## Variants
/// - `Status`: Contains a `ModbusId` and a `VfdStatus`, representing the status response.
/// - `None`: Represents an empty or uninitialized response.
pub enum SoftResponse {
    Status(ModbusId, VfdStatus),
    None,
}

impl TryInto<[u8; 8]> for SoftResponse {
    type Error = ();
    fn try_into(self) -> Result<[u8; 8], Self::Error> {
        if let SoftResponse::Status(id, status) = self {
            let mut response = [id.into(), 2, 3, 0, 0, 0, 0, 0];
            match status {
                VfdStatus::Run(r) => {
                    response[4] = ((r & 0x7f00) >> 8) as u8;
                    response[5] = (r & 0x00ff) as u8;
                    if r < 0 {
                        response[3] = 1;
                    }
                    let crc = crc16(&response[..6]);
                    response[6] = ((crc & 0xff00) >> 8) as u8;
                    response[7] = (crc & 0x00ff) as u8;
                    Ok(response)
                }
                // TODO: implement STOP status response if speed < mini
                _ => {
                    log::error!("SoftResponse.try_into<[u8]>() status {:?} not yet implemented", self);
                    Err(()) 
                },
            }
        } else {
            log::error!("SoftResponse.try_into<[u8]>() Response type conversion not yet implemented: {:?}", self);
            Err(())
        }
    }
}

impl ResponseFn for SoftResponse {
    fn to_raw(self) -> Option<Vec<u8>> {
        let out: Result<[u8;8], ()> = self.try_into();
        out.ok().map(Vec::from)
    }
}