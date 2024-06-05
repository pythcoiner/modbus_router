use crate::batch::Batch;
use crate::device_template;
use crate::devices::vfd::encoder::{VfdCommands, VfdEncoder};
use crate::devices::vfd::requests::{Dir, VfdRequest, VfdResponse, VfdStatus};
use crate::modbus::ModbusId;
use crate::soft_request::{SoftRequest, SoftResponse};
use crate::traits::device::Device;
use crate::traits::polling::{PollerConnector, PollerMessage};
use crate::traits::routing::RouterConnector;

#[derive(Debug, Clone)]
pub struct VfdBatch {
    cmd: Option<VfdRequest>,
    reference: Option<VfdRequest>,
    status: VfdRequest
}

impl VfdBatch {
    fn new(id: ModbusId) -> Self {
        VfdBatch {
            cmd: None,
            reference: None,
            status: VfdRequest::Status(id),
        }
    }
    
    fn take(&mut self) -> Self {
        let out = self.clone();
        self.cmd = None;
        self.reference = None;
        out
    }
    
    fn handle_request(&mut self, request: SoftRequest, device_id: ModbusId) {
        match request {
            SoftRequest::Run(id, r) => {
                let cmd = if r != 0 && id == device_id {
                    if r > 0 {
                        VfdRequest::Cmd(device_id, Dir::Fw)
                    } else {
                        VfdRequest::Cmd(device_id, Dir::Rv)
                    }
                } else {
                    VfdRequest::Stop(device_id)
                };
                let ref_value = match &cmd {
                    VfdRequest::Cmd(_, _) => { (r.abs() & i16::MAX) as u16 }
                    VfdRequest::Stop(_) => { 0 }
                    _ => { panic!("impossible")}
                };
                let reference = VfdRequest::Ref(device_id, ref_value);
                
                if self.cmd.is_none() && self.reference.is_none() {
                    self.reference = Some(reference);
                    self.cmd = Some(cmd);
                }
            }
            SoftRequest::Stop(id) => {
                if id == device_id  && (self.cmd.is_none() && self.reference.is_none()){
                    self.cmd = Some(VfdRequest::Stop(device_id));
                    self.reference = Some(VfdRequest::Ref(device_id, 0));
                }
            }
            _ => { panic!("this request should have been filtered out before!")}
        }
    }
    
    fn retry_request(&mut self, request: VfdRequest) {
        match request {
            VfdRequest::Status(_) => {}
            VfdRequest::Cmd(_, _) |
            VfdRequest::Stop(_) => { 
                if self.cmd.is_none() {
                    self.cmd = Some(request)
                }
            }
            VfdRequest::Ref(_, _) => { 
                if self.reference.is_none() {
                    self.reference = Some(request)
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Vfd {
    id: ModbusId,
    commands: VfdCommands,
    status: VfdStatus,
    batch: VfdBatch,
    router: Option<RouterConnector<SoftRequest, SoftResponse>>,
    poller: Option<PollerConnector<VfdRequest, VfdResponse>>,
    auto_update: bool,
    poll_status: bool,
}

unsafe impl Send for Vfd{}

impl Vfd {
    pub fn new(id: ModbusId, commands: VfdCommands, poll_status: bool) -> Self {
        Vfd {
            id,
            commands,
            status: VfdStatus::None,
            batch: VfdBatch::new(id),
            router: None,
            poller: None,
            auto_update: false,
            poll_status,
        }
    }

    /// Starts the Vfd run loop in a new thread.
    pub fn start(mut self) {
        log::debug!("Vfd.start()");
        tokio::spawn(async move {
            self.run().await;
        });
        log::debug!("Vfd.start() started!");
    }
}

impl Device<SoftRequest, SoftResponse, VfdRequest, VfdResponse> for Vfd
{

    type Encoder = VfdEncoder;

    device_template!(VfdRequest, VfdResponse);

    fn send_batch(&mut self) {
        log::debug!("Vfd.send_batch()");
        if self.is_device_connected() {
            let vfd_batch = self.batch.take();
            let mut batch = Batch::new(self.id, Box::new(VfdEncoder::new(self.commands)));
            if let Some(req) = vfd_batch.cmd {
                batch.push(req);
            }
            if let Some(req) = vfd_batch.reference {
                batch.push(req);
            }
            if self.poll_status {
                batch.push(vfd_batch.status);
            }
            log::debug!("Vfd.send_batch() batch: {:?}", batch);
            if self.poller.as_mut().unwrap().sender.send(batch).is_err() {
                log::error!("Cannot send batch");
            }
            
        } else {
            panic!("Router not connected!");
        }
    }


    fn handle_external_request(&mut self, request: SoftRequest) {
        log::debug!("Device.handle_external_request({:?}) ", request);
        match request {
            SoftRequest::Status(id) => {
                log::debug!("Device.handle_external_request() status request received!");
                if id == self.id {
                    self.send_external_response(SoftResponse::Status(self.id, self.status));
                } else {
                    log::error!("Device.handle_external_request() id {:?} and {:?} does not matches!", id, self.id);
                }
            }
            _ => { self.batch.handle_request(request, self.id,)}
        }
    }

    fn handle_device_response(&mut self, response: VfdResponse) {
        log::debug!("Vfd.handle_device_response({:?})", response);
        match response {
            // if command or ref fail, re-send on next batch
            VfdResponse::Fail(r) => {
                match r {
                    VfdRequest::Status(_) => {}
                    _ => {self.batch.retry_request(r)}
                }
            }
            // update status
            VfdResponse::Status(status) => {
                self.status = status;
                if self.auto_update {
                    self.send_external_response(SoftResponse::Status(self.id, status));
                }
            }
            _ => {}
        }
        
        // auto update
    }
}