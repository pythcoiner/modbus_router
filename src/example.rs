use chrono::Local;
use colored::Colorize;
use std::{env, io};
use std::time::Duration;
use serial_thread::serial::Baud115200;
use lib::devices::joystick::device::{Joystick, JoystickType};
use lib::devices::joystick::requests::{JoystickRequest, JoystickResponse};
use lib::devices::vfd::device::Vfd;
use lib::devices::vfd::encoder::{FRECON, MEGMEET, VfdCommands};
use lib::devices::vfd::requests::{VfdRequest, VfdResponse};
use lib::poller::ModbusPoller;
use lib::soft_request::{SoftRequest, SoftResponse};
use lib::router::StdRouter;
use lib::traits::device::Device;

fn vfd(id: u8, vfd: VfdCommands, router: &mut StdRouter<SoftRequest, SoftResponse>, poller: &mut ModbusPoller<VfdRequest, VfdResponse>, list: &mut Vec<Vfd>) {
    let mut vfd_list = Vfd::new(id.into(), vfd, false);
    vfd_list.connect_poller(poller);
    vfd_list.connect_router(router);

    list.push(vfd_list);
}

fn joystick(id: u8, joystick_type: JoystickType,  router: &mut StdRouter<SoftRequest, SoftResponse>, poller: &mut ModbusPoller<JoystickRequest, JoystickResponse>) -> Joystick {
    let mut joy = Joystick::new(id.into(), joystick_type);
    joy.connect_poller(poller);
    joy.connect_router(router);

    joy
}

#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();

    let router_level = if args.len() < 2 {
        log::LevelFilter::Error
    } else if args[1].to_lowercase() == *"debug" {
        log::LevelFilter::Debug
    } else if args[1].to_lowercase() == *"error" {
        log::LevelFilter::Error
    } else if args[1].to_lowercase() == *"info" {
        log::LevelFilter::Info
    } else {
        log::LevelFilter::Error
    };

    let serial_level = if args.len() < 3 {
        log::LevelFilter::Error
    } else if args[2].to_lowercase() == *"debug" {
        log::LevelFilter::Debug
    } else if args[2].to_lowercase() == *"error" {
        log::LevelFilter::Error
    } else if args[2].to_lowercase() == *"info" {
        log::LevelFilter::Info
    } else {
        log::LevelFilter::Error
    };

    let port0 = if args.len() < 4 {
        "/dev/ttyXR6"
    } else {
        &args[3]
    };

    let port1 = if args.len() < 5 {
        "/dev/ttyXR7"
    } else {
        &args[4]
    };

    let port2 = if args.len() < 6 {
        "/dev/ttyXR2"
    } else {
        &args[5]
    };

    let port3 = if args.len() < 7 {
        "/dev/ttyXR3"
    } else {
        &args[6]
    };

    let port4 = if args.len() < 8 {
        "/dev/ttyXR4"
    } else {
        &args[7]
    };

    let verbose_log = false;
    fern::Dispatch::new()
        .format(move |out, message, record| {
            let color = match record.level() {
                log::Level::Error => "red",
                log::Level::Warn => "yellow",
                log::Level::Info => "green",
                log::Level::Debug => "blue",
                log::Level::Trace => "magenta",
            };

            let file = record.file();
            let line = record.line();
            let mut file_line = "".to_string();

            if let Some(f) = file {
                file_line = format!(":{}", f);
                if let Some(l) = line {
                    file_line = format!("{}:{}", file_line, l);
                }
            }
            let formatted = if verbose_log {
                format!(
                    "[{}][{}{}][{}] {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.target(),
                    file_line,
                    record.level(),
                    message
                )
            } else {
                format!("[{}][{}] {}", record.level(), record.target(), message)
            };
            out.finish(format_args!("{}", formatted.color(color)))
        })
        .level(router_level)
        .level_for("serial_thread", serial_level)
        // .level_for("modbus_router", log::LevelFilter::Error)
        .chain(io::stderr())
        .apply()
        .unwrap();

    let mut router = StdRouter::new(io::stdin(), io::stdout());

    // Serial port #0
    let poller_0 = {
        let mut poller = ModbusPoller::new(port0, Baud115200, Some(1), Some(1), Some(5));
    
        joystick(0x05, JoystickType::Joystick, &mut router, &mut poller).start();
        poller
    };
    poller_0.start();
    
    // Serial port #1
    let poller_1 = {
        let mut poller = ModbusPoller::new(port1, Baud115200, Some(1), Some(1), Some(5));
    
        joystick(0x06, JoystickType::Joystick, &mut router, &mut poller).start();
        poller
    };
    poller_1.start();

    // Serial port #2
    let poller_2 = {
        let mut poller = ModbusPoller::new(port2, Baud115200, Some(1), Some(1), Some(5));
        let mut vfd_list: Vec<Vfd> = Vec::new();

        vfd(10, MEGMEET, &mut router, &mut poller, &mut vfd_list);
        vfd(11, MEGMEET, &mut router, &mut poller, &mut vfd_list);
        vfd(60, MEGMEET, &mut router, &mut poller, &mut vfd_list);
        vfd(61, MEGMEET, &mut router, &mut poller, &mut vfd_list);

        for vfd in vfd_list {
            vfd.start()
        }

        poller
    };
    poller_2.start();

    // Serial port #3
    let poller_3 = {
        let mut poller = ModbusPoller::new(port3, Baud115200, Some(3), Some(6), Some(6));
        let mut vfd_list: Vec<Vfd> = Vec::new();

        vfd(12, FRECON, &mut router, &mut poller, &mut vfd_list);
        vfd(20, FRECON, &mut router, &mut poller, &mut vfd_list);
        vfd(21, FRECON, &mut router, &mut poller, &mut vfd_list);
        vfd(26, FRECON, &mut router, &mut poller, &mut vfd_list);
        vfd(27, FRECON, &mut router, &mut poller, &mut vfd_list);

        for vfd in vfd_list {
            vfd.start()
        }

        poller
    };
    poller_3.start();

    // Serial port #4
    let poller_4 = {
        let mut poller = ModbusPoller::new(port4, Baud115200, Some(3), Some(6), Some(6));
        let mut vfd_list: Vec<Vfd> = Vec::new();

        vfd(30, FRECON, &mut router, &mut poller, &mut vfd_list);
        vfd(31, FRECON, &mut router, &mut poller, &mut vfd_list);
        vfd(40, MEGMEET, &mut router, &mut poller, &mut vfd_list);
        vfd(43, FRECON, &mut router, &mut poller, &mut vfd_list);
        vfd(50, FRECON, &mut router, &mut poller, &mut vfd_list);
        vfd(51, MEGMEET, &mut router, &mut poller, &mut vfd_list);

        for vfd in vfd_list {
            vfd.start()
        }

        poller
    };
    poller_4.start();

    tokio::time::sleep(Duration::from_millis(100)).await;
    // start router
    router.start().await;
}
