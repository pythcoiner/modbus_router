use std::fmt::Debug;
use serial_thread::SerialMessage;
use crate::modbus::ModbusId;
use crate::traits::device_encoder::DeviceEncoder;


#[derive(Debug)]
pub struct Batch<DeviceRequest, DeviceResponse>
{
    encoder: Box<dyn DeviceEncoder<DeviceRequest, DeviceResponse>>,
    requests: Vec<DeviceRequest>,
    current_request: Option<DeviceRequest>,
    pub(crate) id: ModbusId,
}

impl<DeviceRequest, DeviceResponse> Batch<DeviceRequest, DeviceResponse>
where
    DeviceRequest: Debug + Clone + Copy,
    DeviceRequest: Debug + Clone + Copy,
{
    pub fn new(id: ModbusId, encoder: Box<dyn DeviceEncoder<DeviceRequest, DeviceResponse>>) -> Self {
        Batch {
            encoder,
            requests: vec![],
            current_request: None,
            id,
        }
    }

    #[allow(clippy::should_implement_trait)]
    /// Yield the next request, return None if no requests remains or if the current request
    /// have not yet been answered.
    pub fn next(&mut self) -> Option<SerialMessage> {
        if !self.requests.is_empty() && self.current_request.is_none() {
            let request = self.requests.pop().unwrap();
            self.current_request = Some(request);
            self.encoder.request_to_serial(request)
        } else {
            None
        }
    }

    /// Return true if no request remaining and current request is None.
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty() && self.current_request.is_none()
    }

    /// Return true if the current request have been answered.
    pub fn is_complete(&self) -> bool {
        self.current_request.is_none()
    }

    /// Try to handle the response, return None if the response is not related to the current 
    /// request (or if no current request)
    pub fn handle_response(&mut self, msg: SerialMessage) -> Option<DeviceResponse> {
        if self.current_request.is_none() {
            log::error!("Batch.handle_response() => cannot decode response, as there is no current request!");
            None
        }else if let Some(m) = self.encoder.filter_response(msg) {
            if let Some(request) = self.current_request.take() {
                Some(self.encoder.serial_to_response(m, request, self.id))
            } else {
                panic!("Cannot handle response if no current request")
            }
            
        } else {
            None
        }
    }
    
    pub fn push(&mut self, request: DeviceRequest) {
        log::debug!("Vfd.Batch.push({:?}", &request);
        self.requests.push(request);
        
    }
}