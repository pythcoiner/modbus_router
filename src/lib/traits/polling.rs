use std::fmt::Debug;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use serial_thread::{Mode, SerialInterface, SerialMessage};
use tokio::time::sleep;
use crate::batch::Batch;
use crate::modbus::ModbusId;

#[derive(Debug)]
pub enum PollerMessage<DeviceResponse> {
    Poll,
    Response(DeviceResponse)
}

#[allow(unused)]
#[derive(Debug)]
/// A connector for a poller in a VFD system.
///
/// Holds the sender and receiver channels for managing the polling mechanism.
pub struct PollerConnector<DeviceRequest, DeviceResponse>

{
    pub sender: Sender<Batch<DeviceRequest, DeviceResponse>>,
    pub receiver: Receiver<PollerMessage<DeviceResponse>>,
}

pub trait Polling<DeviceRequest, DeviceResponse>
where 
    DeviceRequest: Debug + Clone + Copy + Send,
    DeviceResponse: Debug + Clone + Copy + Send,
{

    // log::debug!("Poller: batch received => {:?}", batch);
    fn rcv_batch(&mut self) -> Option<Batch<DeviceRequest, DeviceResponse>>;

    fn get_connector(&mut self, id: ModbusId) -> Option<PollerConnector<DeviceRequest, DeviceResponse>>;
    fn port_name(&self) -> &str;
    fn devices_count(&self) -> usize;

    //self.serial_port.take()
    fn take_serial(&mut self) -> Option<SerialInterface>;
    fn get_timeout(&self) -> Option<Duration>;
    fn get_frame_silence(&self) -> Option<Duration>;
    fn get_device_silence(&self) -> Option<Duration>;
    fn send_msg(&mut self, msg: SerialMessage);

    // self.serial_receiver.try_recv()
    fn receive_msg(&mut self) -> Option<SerialMessage>;


    fn devices_ids(&self) -> Vec<ModbusId>;

    fn send_to_device(&mut self, id: ModbusId, msg: PollerMessage<DeviceResponse>);

    // log::debug!("polling device {}", {let i: u8 = (*axis_id).into(); i});
    // self.send_to_device(self.poll_message);
    fn poll(&mut self, id: ModbusId) {
        self.send_to_device(id, PollerMessage::Poll)
    }


    /// Runs the device polling logic.
    ///
    /// This async function is responsible for initiating and managing the polling process.
    /// It handles setting up the serial connection, sending and receiving batches of requests,
    /// and processing responses.
    ///
    /// It continually runs in a loop, managing the flow of information
    /// between the serial interface and the devices.
    #[allow(async_fn_in_trait)]
    async fn run(&mut self) {
        log::info!(
            "Poller {} Started, {} devices.",
            self.port_name(),
            self.devices_count()
        );

        // start serial interface
        if let Some(mut serial) = self.take_serial() {
            tokio::spawn(async move {
                serial.start().await;
            });
        } else {
            log::error!("SerialInterface({}) missing",self.port_name())
        }

        if let Some(timeout) = self.get_timeout() {
            log::info!("{} => SetTimeout({:?})",self.port_name(), &timeout);
            self.send_msg(SerialMessage::SetTimeout(timeout));
        }

        self.send_msg(SerialMessage::Connect);

        // wait for serial port to connect 
        loop {
            if let Some(SerialMessage::Connected(connected)) = self.receive_msg() {
                log::debug!("{:?}", SerialMessage::Connected(connected));
                if connected {
                    break;
                } else {
                    log::error!("Cannot connect to serial port {}", self.port_name());
                    return;
                }
            } else {
                sleep(Duration::from_nanos(10)).await;
            }
        }

        log::info!("{} connected!", self.port_name());

        self.send_msg(SerialMessage::SetMode(Mode::MasterStream));
        log::info!("{} => SetMode(Mode::MasterStream)!", self.port_name());
        
        log::info!("Poller => Start polling {} for {} devices", self.port_name(), self.devices_ids().len());
        loop {
            // poll each device
            for device_id in self.devices_ids() {
                self.poll(device_id);

                //  wait for batch
                let mut batch: Batch<DeviceRequest, DeviceResponse>;
                loop {
                    if let Some(b) = self.rcv_batch() {
                        batch = b;
                        break;
                    } else {
                        sleep(Duration::from_nanos(10)).await;
                    }
                }

                while !batch.is_empty() {
                    let next_request = batch.next();
                    if let Some(request) = next_request {
                        self.send_msg(request);
                        while !batch.is_complete() {
                            let serial_response = self.receive_msg();
                            if let Some(r) = serial_response {
                                let to_send = batch.handle_response(r);
                                if let Some(send) = to_send {
                                    self.send_to_device(batch.id, PollerMessage::Response(send));
                                    if let Some(silence) = self.get_frame_silence() {
                                        sleep(silence).await;
                                    }
                                    break
                                }
                            } else {
                                sleep(Duration::from_nanos(10)).await;
                            }
                        }
                    } else {
                        if let Some(silence) = self.get_device_silence() {
                            sleep(silence).await;
                        }
                        break;
                    }
                }
                sleep(Duration::from_nanos(10)).await;
            }
        }
    }
}
