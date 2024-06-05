use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{Stdin, Stdout, Write};
use std::sync::mpsc::{channel, Receiver, Sender};
use modbus_core::rtu::crc16;
use crate::modbus::ModbusId;
use crate::traits::request::{RequestFn, ResponseFn};
use crate::traits::routing::{RouterConnector, Routing};
use crate::async_stdin::stdin_channel;
use tokio::sync::mpsc::Receiver as StdinChannel;

pub const FRAME_LENGTH: usize = 8;

#[allow(unused)]
#[derive(Debug)]
/// Manages routing of PLC requests and responses between external processes and modbus devices.
///
/// `StdRouter` listens for incoming requests from an external process via `stdin` and dispatches
/// these requests to the appropriate modbus devices based on their Modbus ID. It also handles
/// broadcasting requests and dispatching responses back to the external process through `stdout`.
pub struct StdRouter<Request, Response> {
    stdin: StdinChannel<u8>,
    buff_stdin: Vec<u8>,
    stdout: Stdout,
    receiver: Receiver<Response>,
    connector: Sender<Response>,
    senders: HashMap<ModbusId, Sender<Request>>,
}

impl<Request, Response> StdRouter<Request, Response>
    where
        Request: RequestFn + Copy + 'static,
        Response: ResponseFn + Copy + 'static,
{
    /// Constructs a new `StdRouter`.
    ///
    /// Initializes the router with the given `stdin` and `stdout` handles for communication.
    ///
    /// # Arguments
    /// * `stdin` - Standard input for receiving external requests.
    /// * `stdout` - Standard output for sending responses back to the external process.
    pub fn new(stdin: Stdin, stdout: Stdout) -> Self {
        let (connector, receiver) = channel();
        let stdin = stdin_channel(stdin);
        StdRouter {
            stdin,
            buff_stdin: vec![],
            stdout,
            receiver,
            connector,
            senders: Default::default(),
        }
    }

    pub fn try_read(&mut self) -> Option<Vec<u8>> {
        if let Ok(byte) = self.stdin.try_recv() {
            self.buff_stdin.push(byte);
            self.try_decode_read()
        } else {
            None
        }
    }
    
    /// Scan a FRAME_LENGTH long window in `buff_stdin` until finding a valid frame (CRC match)
    /// this to avoid having an offset in stdin stream.
    fn try_decode_read(&mut self) -> Option<Vec<u8>> {
        while self.buff_stdin.len() > 7 {
            if Self::check_crc(&self.buff_stdin[..FRAME_LENGTH]) {
                log::debug!("StdRouter extract frame: {:?}", &self.buff_stdin[..FRAME_LENGTH]);
                let out = self.buff_stdin.iter().take(FRAME_LENGTH).cloned().collect();
                self.buff_stdin.drain(0..8);
                return Some(out)
            }
            self.buff_stdin.remove(0);
        }
        None
    }
    
    fn check_crc(frame: &[u8]) -> bool {
        if frame.len() == 8 {
            let crc = crc16(&frame[..frame.len()-2]);
            let expected_crc = [((crc & 0xff00) >> 8) as u8, (crc & 0x00ff) as u8];
            expected_crc == frame[frame.len()-2..]
        } else {
            false
        }
        
    }

    #[allow(unused)]
    /// Starts the run loop of the `Router` in a new thread.
    pub async fn start(mut self) {
        self.run().await;
    }
    
}

#[allow(unused)]
impl<Request, Response> Routing<Request, Response> for StdRouter<Request, Response>
    where
        Request: RequestFn + Copy + 'static,
        Response: ResponseFn + Copy + 'static,
{
    /// Retrieves or creates a new `RouterConnector` for a specific Modbus ID.
    ///
    /// # Arguments
    /// * `id` - The `ModbusId` for which to create or retrieve the connector.
    ///
    /// # Returns
    /// * `Ok(RouterConnector)` if a new connector was created or retrieved successfully.
    /// * `Err(Error)` if a connector for the specified ID already exists.
    fn get_connector(&mut self, id: ModbusId) -> Option<RouterConnector<Request, Response>> {
        if let std::collections::hash_map::Entry::Vacant(e) = self.senders.entry(id) {
            let (sender, receiver) = channel();
            e.insert(sender);

            Some(RouterConnector {
                sender: self.connector.clone(),
                receiver,
            })
        } else {
            None
        }
    }

    /// Routes a Request to the respective device.
    ///
    /// # Arguments
    /// * `request` - A `PlcRequest` to be sent to the corresponding VFD axis.
    fn transmit_request(&mut self, request: Request) {
        if let Some(sender) = self.senders.get_mut(&request.id()) {
            log::debug!("StdRouter.transmit_request({:?}) to {:?}", request, request.id());
            let _ = sender.send(request);
        } else {
            log::error!("StdRouter.transmit_request() no receiver for id {:?}", &request.id())
        }
    }

    /// Routes a response from a device to the `stdout`
    fn transmit_response(&mut self, response: Vec<u8>) {
        log::debug!("StdRouter.transmit_response({:?}) to stdout", &response);
        let _ = self.stdout.write_all(response.as_slice());
    }

    /// Used for logging
    fn devices_count(&self) -> usize {
        self.senders.len()
    }
    
    /// Return a list of devices ids, used for convert a broadcast request into devices requests.
    fn devices_ids(&self) -> Vec<ModbusId> {
        self.senders.keys().cloned().collect()
    }

    fn try_receive_request(&mut self) -> Option<Vec<u8>> {
        self.try_read()
    }

    fn try_receive_response(&mut self) -> Option<Response> {
        self.receiver.try_recv().ok()
    }
    
}
