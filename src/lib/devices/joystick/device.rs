use crate::batch::Batch;
use crate::device_template;
use crate::devices::joystick::encoder::JoystickEncoder;
use crate::devices::joystick::requests::{JoystickRequest, JoystickResponse, JoystickStatus};
use crate::modbus::ModbusId;
use crate::soft_request::{SoftRequest, SoftResponse};
use crate::traits::device::Device;
use crate::traits::polling::{PollerConnector, PollerMessage};
use crate::traits::routing::RouterConnector;


#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum JoystickType {
    Joystick,
    JoystickWithThumb,
}

#[derive(Debug)]
pub struct Joystick {
    id: ModbusId,
    joystick_type: JoystickType,
    status: JoystickStatus,
    router: Option<RouterConnector<SoftRequest, SoftResponse>>,
    poller: Option<PollerConnector<JoystickRequest, JoystickResponse>>,
}

unsafe impl Send for Joystick{}

impl Joystick {
    pub fn new(id: ModbusId, joystick_type: JoystickType) -> Self {
        Joystick {
            id,
            joystick_type,
            status: JoystickStatus::None,
            router: None,
            poller: None,
        }
    }
    
    fn update_status(&mut self, status: JoystickStatus) {
        // TODO
        if status != self.status {
            println!("Joystick {:?}: {:?}", self.id, status);
            self.status = status;
        }
    }

    /// Starts the Vfd run loop in a new thread.
    pub fn start(mut self) {
        tokio::spawn(async move {
            self.run().await;
        });
    }
}

impl Device<SoftRequest, SoftResponse, JoystickRequest, JoystickResponse> for Joystick
{
    type Encoder = JoystickEncoder;

    device_template!(JoystickRequest, JoystickResponse);

    fn send_batch(&mut self) {
        if self.is_device_connected() {
            let mut batch: Batch<JoystickRequest, JoystickResponse> =
                Batch::new(self.id(), Box::new(JoystickEncoder::new(self.joystick_type)));
            batch.push(JoystickRequest::Status(self.id(), self.joystick_type));
            
            if self.poller.as_mut().unwrap().sender.send(batch).is_err() {
                log::debug!("Joystick: cannot send batch");
            }
        }
    }

    fn handle_external_request(&mut self, _request: SoftRequest) {
        /* Drop every received request! */
    }

    fn handle_device_response(&mut self, response: JoystickResponse) {
        match response {
            JoystickResponse::Status(status) => {self.update_status(status)}
            JoystickResponse::Fail(_) => {/* TODO: handle lost request counting */}
        }
    }
}