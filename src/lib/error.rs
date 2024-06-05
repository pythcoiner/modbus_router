#[allow(unused)]
#[derive(Debug)]
pub enum VfdError {
    ChannelAllReadyConnected,
    WrongFrameLength,
    WrongCrc,
    WrongFrameType,
    WrongFunctionType,
    WrongRefValue,
    WrongRefSign,
    WrongModbusId,
    NotImplemented,
}
