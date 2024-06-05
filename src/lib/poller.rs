use serial_thread::serial::BaudRate;
use serial_thread::{SerialInterface, SerialMessage};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use crate::batch::Batch;
use crate::modbus::ModbusId;
use crate::traits::polling::{PollerConnector, PollerMessage, Polling};

pub struct ModbusPoller<DeviceRequest, DeviceResponse>
    where
        DeviceRequest: Copy + 'static,
        DeviceResponse: Copy + 'static,
{
    port: String,
    serial_port: Option<SerialInterface>,
    serial_sender: Sender<SerialMessage>,
    serial_receiver: Receiver<SerialMessage>,
    receiver: Receiver<Batch<DeviceRequest, DeviceResponse>>,
    connector: Sender<Batch<DeviceRequest, DeviceResponse>>,
    senders: HashMap<ModbusId, Sender<PollerMessage<DeviceResponse>>>,
    frame_silence: Option<u64>,
    device_silence: Option<u64>,
    timeout: Option<u64>,
}


/// `Poller` manages communication with one or more devices. It handles sending and receiving
/// Modbus messages over a serial interface, processing these messages, and maintaining
/// the state of each device.
///
/// Fields:
/// - `port`: The name of the serial port.
/// - `serial_port`: `SerialInterface` for the serial communication.
/// - `serial_sender`: Channel sender for sending `SerialMessage` to the serial thread.
/// - `serial_receiver`: Channel receiver for receiving `SerialMessage` from the serial thread.
/// - `receiver`: Receiver for `Batch` messages containing batches of VFD requests.
/// - `connector`: Sender for transmitting `Batch` messages.
/// - `senders`: A map of `ModbusId` to senders for `Response` messages.
/// - `pending_request`: Optional `Request` representing a request awaiting a response.
/// - `frame_silence`: Optional duration of silence required after sending each frame.
/// - `device_silence`: Optional duration of silence required after communicating with each device.
/// - `timeout`: Optional timeout duration for the serial communication.
#[allow(unused)]
impl<DeviceRequest, DeviceResponse> ModbusPoller<DeviceRequest, DeviceResponse>
    where
        DeviceRequest: Debug + Clone + Copy + Send + 'static,
        DeviceResponse: Debug + Clone + Copy + Send + 'static,
{

    /// Constructs a new `Poller`.
    ///
    /// Parameters:
    /// - `port`: The name of the serial port.
    /// - `bauds`: baud rate.
    /// - `frame_silence`: Optional duration of silence after each frame.
    /// - `device_silence`: Optional duration of silence after each device communication.
    /// - `timeout`: Optional timeout for the serial communication.
    pub fn new(port: &str, 
               bauds: BaudRate, 
               frame_silence: Option<u64>, 
               device_silence: Option<u64>, 
               timeout: Option<u64>) -> Self {
        let (poller_sender, serial_receiver) = channel();
        let (serial_sender, poller_receiver) = channel();

        let mut serial_port = SerialInterface::new()
            .unwrap()
            .path(port.to_string())
            .receiver(serial_receiver)
            .sender(serial_sender)
            .bauds(bauds);

        let (connector, receiver) = channel();

        ModbusPoller {
            port: port.to_string(),
            serial_port: Some(serial_port),
            serial_sender: poller_sender,
            serial_receiver: poller_receiver,
            receiver,
            connector,
            senders: HashMap::new(),
            frame_silence,
            device_silence,
            timeout,
        }
    }

    /// Starts the run loop of the `Router` in a new thread.
    pub fn start(mut self) {
        tokio::spawn(async move {
            self.run().await;
        });
    }
}

impl<DeviceRequest, DeviceResponse> Polling<DeviceRequest, DeviceResponse> for ModbusPoller<DeviceRequest, DeviceResponse>
    where
        DeviceRequest: Debug + Clone + Copy + Send,
        DeviceResponse: Debug + Clone + Copy + Send,
{

    fn rcv_batch(&mut self) -> Option<Batch<DeviceRequest, DeviceResponse>> {
        self.receiver.try_recv().ok()
    }

    fn get_connector(&mut self, id: ModbusId) -> Option<PollerConnector<DeviceRequest, DeviceResponse>> {
        if let std::collections::hash_map::Entry::Vacant(e) = self.senders.entry(id) {
            let (sender, receiver) = channel();
            e.insert(sender);

            Some(PollerConnector {
                sender: self.connector.clone(),
                receiver,
            })
        } else {
            None
        }
    }

    fn port_name(&self) -> &str {
        &self.port
    }

    fn devices_count(&self) -> usize {
        self.senders.len()
    }

    fn take_serial(&mut self) -> Option<SerialInterface> {
        self.serial_port.take()
    }

    fn get_timeout(&self) -> Option<Duration> {
        self.timeout.map(Duration::from_millis)
    }

    fn get_frame_silence(&self) -> Option<Duration> {
        self.frame_silence.map(Duration::from_millis)
    }

    fn get_device_silence(&self) -> Option<Duration> {
        self.device_silence.map(Duration::from_millis)
    }

    fn send_msg(&mut self, msg: SerialMessage) {
        log::debug!("ModbusPoller.send_msg() {:?} ", msg);
        if let Err(e) = self.serial_sender.send(msg) {
            log::debug!("{:?}", e);
        }
    }

    fn receive_msg(&mut self) -> Option<SerialMessage> {
        match self.serial_receiver.try_recv() {
            Ok(msg) => {
                log::debug!("ModbusPoller.receive_msg() {:?} ", msg);
                Some(msg)
            }
            Err(_e) => {
                // log::debug!("{:?}", e);
                None
            }
        }
    }

    fn devices_ids(&self) -> Vec<ModbusId> {
        self.senders.clone().into_keys().collect()
    }

    fn send_to_device(&mut self, id: ModbusId, msg: PollerMessage<DeviceResponse>) {
        if let Some(sender) = self.senders.get_mut(&id) {
            log::debug!("ModbusPoller.send_to_device() {:?} to device {} ", msg, {let i: u8 = id.into(); i});
            let _ = sender.send(msg);
        } else {
            log::debug!("Sender for device {} missing!", {
                let i: u8 = id.into();
                i
            })
        }
    }

}
