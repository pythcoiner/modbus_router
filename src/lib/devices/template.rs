#[macro_export] 
macro_rules! device_template {
    ($Request:ty, $Response:ty) => {
        fn set_poller(&mut self, connector: PollerConnector<$Request, $Response>) {
        self.poller = Some(connector);
    }

    fn set_router(&mut self, connector: RouterConnector<SoftRequest, SoftResponse>) {
        self.router = Some(connector);
    }

    fn id(&self) -> ModbusId {
        self.id
    }

    fn is_external_connected(&self) -> bool {
        self.router.is_some()
    }

    fn is_device_connected(&self) -> bool {
        self.poller.is_some()
    }

    fn read_external_request(&mut self) -> Option<SoftRequest> {
        if self.is_external_connected() {
            self.router.as_mut().unwrap().receiver.try_recv().ok()
        } else {
            panic!("Router not connected!");
        }
    }

    fn send_external_response(&mut self, response: SoftResponse) {
        log::debug!("Device.send_external_response({:?})", response);    
        if self.is_external_connected() {
            if self.router.as_mut().unwrap().sender.send(response).is_err() {
                log::debug!("Cannot send response: {:?}", response);
            }
        } else {
            panic!("Router not connected!");
        }
    }

    fn read_device_response(&mut self) -> Option<PollerMessage<$Response>> {
        if self.is_device_connected() {
            self.poller.as_mut().unwrap().receiver.try_recv().ok()
        } else {
            panic!("Router not connected!");
        }
    }
    };
}