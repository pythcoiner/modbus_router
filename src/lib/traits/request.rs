use std::fmt::Debug;
use crate::modbus::ModbusId;


pub trait RequestFn: Debug + Clone + Copy + Send{
    fn from(raw: Vec<u8>) -> Option<Box<Self>>;
    fn id(&self) -> ModbusId;
    fn new_id(&self, id: ModbusId) -> Box<Self>;
}

pub trait ResponseFn: Debug + Clone + Copy + Send {
    fn to_raw(self) -> Option<Vec<u8>>;
}