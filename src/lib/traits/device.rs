use std::fmt::Debug;
use std::time::Duration;
use tokio::time::sleep;
use crate::modbus::ModbusId;
use crate::traits::device_encoder::DeviceEncoder;
use crate::traits::request::{RequestFn, ResponseFn};
use crate::traits::polling::{PollerConnector, PollerMessage, Polling};
use crate::traits::routing::{RouterConnector, Routing};

pub trait Device<Request, Response, DeviceRequest, DeviceResponse>
    where
        Request: RequestFn,
        Response: ResponseFn,
        DeviceRequest: Clone + Copy,
        DeviceResponse: Clone + Copy,
{
    type Encoder: DeviceEncoder<DeviceRequest, DeviceResponse>;

    /// Connects Device to a given Poller.
    ///
    /// Panics if there is already a poller connected.
    ///
    /// # Arguments
    /// * `poller` - A `Poller` to be connected to the Device.
    fn connect_poller(&mut self, poller: &mut impl Polling<DeviceRequest, DeviceResponse>) 
    where
        DeviceRequest: Debug + Clone + Copy + Send,
        DeviceResponse: Debug + Clone + Copy + Send,
    {
        if !self.is_device_connected() {
            if let Some(conn) = poller.get_connector(self.id()) {
                self.set_poller(conn);
            } else {
                panic!("Poller already connected!");
            }
        } else {
            panic!("Poller already connected!");
        }
    }

    /// Connects Device to a given Router.
    ///
    /// Panics if there is already a router connected.
    ///
    /// # Arguments
    /// * `router` - A `Router` to be connected to the Device.
    fn connect_router(&mut self, router: &mut impl Routing<Request, Response>) {
        if !self.is_external_connected() {
            if let Some(conn) = router.get_connector(self.id()) {
                self.set_router(conn);
            } else {
                panic!("Router already connected!");
            }
        } else {
            panic!("Router already connected!");
        }
    }
    
    fn set_poller(&mut self, connector: PollerConnector<DeviceRequest, DeviceResponse>);
    // self.poller = Some(conn);

    fn set_router(&mut self, connector: RouterConnector<Request, Response>);
    // self.router = Some(conn);

    fn send_batch(&mut self);
    
    fn id(&self) -> ModbusId;
    
    /// Return true if connected to router
    fn is_external_connected(&self) -> bool;
    
    /// Return true if connected to poller
    fn is_device_connected(&self) -> bool;
    
    fn read_external_request(&mut self) -> Option<Request>;
    
    fn send_external_response(&mut self, response: Response);

    fn read_device_response(&mut self) -> Option<PollerMessage<DeviceResponse>>;
    // self.poller.as_mut().unwrap().receiver.try_recv()
    
    fn handle_external_request(&mut self, request: Request);
    fn handle_device_response(&mut self, response: DeviceResponse);
    
    /// Function that continually handles external requests and device responses.
    ///
    /// It processes incoming requests and responses, updates the state, and handles communication
    /// with external and poller.
    #[allow(async_fn_in_trait)]
    async fn run(&mut self) {
        log::info!("Device with id {} Started.", {
        let id: u8 = self.id().into();
        id
    });
        loop {
            if self.is_external_connected() {
                while let Some(request) =  self.read_external_request(){
                    log::debug!("Device::get external request: {:?}", request);
                    self.handle_external_request(request);
                }
            } else {
                panic!("No router!");
            }

            if self.is_device_connected() {
                while let Some(response) =  self.read_device_response(){
                    match response {
                        PollerMessage::Poll => {
                            self.send_batch();
                        }
                        PollerMessage::Response(r) => {
                            self.handle_device_response(r);
                        }
                    }
                }
            } else {
                panic!("No Poller!");
            }

            sleep(Duration::from_nanos(10)).await;
        }
    }
    
}