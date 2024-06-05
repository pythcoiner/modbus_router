use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use tokio::time::sleep;
use crate::modbus::ModbusId;
use crate::traits::request::{RequestFn, ResponseFn};

#[allow(unused)]
#[derive(Debug)]
/// Connector for routing Responses.
///
/// Acts as an interface for a specific device to communicate with a `Router`. It contains
/// channels for sending `Response` and receiving `Request`.
pub struct RouterConnector<Request, Response> {
    pub sender: Sender<Response>,
    pub receiver: Receiver<Request>,
}

pub trait Routing<Request, Response>
    where
        Request: RequestFn,
        Response: ResponseFn,
{
    fn get_connector(&mut self, id: ModbusId) -> Option<RouterConnector<Request, Response>>;
    fn transmit_request(&mut self, request: Request);
    fn transmit_response(&mut self, raw: Vec<u8>);
    fn devices_count(&self) -> usize;
    fn devices_ids(&self) -> Vec<ModbusId>;
    fn try_receive_request(&mut self) -> Option<Vec<u8>>;
    fn try_receive_response(&mut self) -> Option<Response>;

    /// Runs the Router loop, handling incoming requests and responses.
    ///
    /// Continuously reads requests from stream and handles incoming responses from devices,
    /// ensuring continuous communication between external processes and devices.
    #[allow(async_fn_in_trait)]
    async fn run(&mut self) {
        log::info!("Router Started, {} devices.", self.devices_count());
        loop {
            while let Some(data) = self.try_receive_request() {
                self.handle_raw_request(data);
            }

            while let Some(response) = self.try_receive_response() {
                self.handle_response(response);
            }

            sleep(Duration::from_nanos(10)).await;
        }
    }
    /// Handles an incoming request.
    ///
    /// Decodes the request and routes it to the appropriate axis, or handles broadcasting.
    ///
    /// # Arguments
    /// * `request` - A Vec<u8> request.
    fn handle_raw_request(&mut self, raw_request: Vec<u8>) {
        log::debug!("Routing.handle_raw_request({:?})", raw_request);
        let r = Request::from(raw_request.clone());
        if let Some(request) = r {
            match request.id() {
                ModbusId::Id(_) => {
                    self.transmit_request(*request);
                }
                ModbusId::Broadcast => {
                    let ids = self.devices_ids();
                    for id in ids {
                        self.transmit_request(*request.new_id(id));
                    }
                }
                ModbusId::Reserved => {}
            }
        }
    }
    /// Handles a PLC response.
    ///
    /// Converts the `PlcResponse` into a byte array and writes it to `stdout`.
    ///
    /// # Arguments
    /// * `response` - A `PlcResponse` to handle.
    fn handle_response(&mut self, response: Response) {
        if let Some(raw) = response.to_raw() {
            log::debug!("Routing.handle_response({:?}) ", response);
            self.transmit_response(raw);
        }
    }

}