use std::collections::HashMap;
use std::time::Duration;

use libusb::{Device, DeviceHandle, DeviceList};

use crate::device_error::DeviceError;

static SETUP_COMMAND: [u8; 8] = [0x08, 0x02, 0x33, 0x00, 0x24, 0x00, 0x00, 0x00];

lazy_static! {
    static ref LIGHT_MODES: HashMap<&'static str, [u8; 8]> = {
        let mut map = HashMap::new();
        map.insert("off", [0x08, 0x02, 0x03, 0x05, 0x00, 0x08, 0x01, 0x00]);
        map.insert("fade", [0x08, 0x02, 0x02, 0x05, 0x32, 0x08, 0x00, 0x00]);
        map.insert("wave", [0x08, 0x02, 0x03, 0x05, 0x32, 0x08, 0x00, 0x00]);
        map.insert("dots", [0x08, 0x02, 0x04, 0x05, 0x32, 0x08, 0x00, 0x00]);
        map.insert("rainbow", [0x08, 0x02, 0x05, 0x05, 0x32, 0x08, 0x00, 0x00]);
        map.insert(
            "explosion",
            [0x08, 0x02, 0x06, 0x05, 0x32, 0x08, 0x00, 0x00],
        );
        map.insert("snake", [0x08, 0x02, 0x09, 0x05, 0x32, 0x08, 0x00, 0x00]);
        map.insert(
            "raindrops",
            [0x08, 0x02, 0x0a, 0x05, 0x32, 0x08, 0x00, 0x00],
        );
        map
    };
}

type DeviceResult<T> = Result<T, DeviceError>;

pub fn list_devices(devices: DeviceList) {
    for mut device in devices.iter() {
        let device_desc = device.device_descriptor().unwrap();

        println!(
            "Bus {:03} Device {:05} ID {:04x}:{:04x}",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id()
        );
    }
}

pub fn get_keyboard(devices: DeviceList, vid: u16, pid: u16) -> DeviceResult<Device> {
    let mut result: DeviceResult<Device> = Err(DeviceError::DeviceNotFound);
    for mut device in devices.iter() {
        let device_desc = device.device_descriptor().unwrap();

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            result = Ok(device);
            println!("{:?}", device_desc);
        }
    }
    return result;
}

pub fn setup_mode(k: &mut Device, mode: &str) {
    //println!("Device found: {:?}", k);
    match k.open() {
        Ok(mut c) => {
            let index: u16 = c.active_configuration().unwrap() as u16;
            println!("usb active configuration index: {:?}", index);

            // send setup command
            match c.write_control(
                0x21,
                9,
                0x300,
                index,
                &SETUP_COMMAND,
                Duration::from_secs(1),
            ) {
                Ok(siz) => println!("transferred: {} bytes", siz),
                Err(e) => {
                    println!("cannot get active config: {}", e);
                    return;
                }
            }

            // set mode
            let ctrl_buff: Option<&[u8; 8]> = LIGHT_MODES.get(mode);
            match ctrl_buff {
                Some(b) => {
                    // send change mode command
                    match c.write_control(0x21, 9, 0x300, index, b, Duration::from_millis(300)) {
                        Ok(_siz) => println!("Ok"),
                        Err(e) => println!("Could not set mode: {}", e),
                    }
                }
                None => println!("Unknown mode '{}'", mode),
            }
        }
        Err(e) => println!("unable to open usb device: {}", e),
    }
}

fn set_row_colors(handle: &mut DeviceHandle, colors: &[[&str; 21]; 6]) {
    // send setup command
    match handle.write_control(
        0x21,
        9,
        0x300 as u16,
        1u16,
        &SETUP_COMMAND,
        Duration::from_secs(2),
    ) {
        Ok(_) => {}
        Err(e) => {
            println!("could not init: {}", e);
            return;
        }
    }

    for r in 0..colors.len() {
        let mut msg: [u8; 64] = [0; 64];
        for c in 0..20 {
            msg[c + 0] /* blue */ = if "red".eq(colors[r][c]) { 0 } else { 0x80 };
            msg[c + 21] /* green */ = 0xff;
            msg[c + 42] /* red */ = 0xff;
        }

        // send setup for row "r"
        let command: [u8; 8] = [0x16, 0, r as u8, 0, 0, 0, 0, 0];
        //println!("setup row: {:?}", command);
        let r_wrote = handle.write_control(
            0x21,
            9u8,
            0x300 as u16,
            1u16 /*index*/,
            &command,
            Duration::from_millis(500),
        );

        match r_wrote {
            Ok(d_siz) => {
                print!("row {:2}: ctrl ok ({} bytes)", r, d_siz);
                // send colors array for row
                match handle.write_interrupt(/*endpoint*/ 1u8, &msg, Duration::from_millis(500)) {
                    Ok(siz) => println!(", data ok ({} bytes)", siz),
                    Err(e) => println!(", data write failure \"{}\"", e),
                }
            }
            Err(e) => println!("Could not set color: {}", e),
        }
    }
}

pub fn set_color(k: &mut Device, colors: &[[&str; 21]; 6]) {
    //println!("Device found: {:?}", k);
    match k.open() {
        Ok(mut handle) => {
            let iface = handle.active_configuration().unwrap();
            println!("usb active configuration index: {:?}", iface);
            match handle.kernel_driver_active(iface) {
                Ok(active) => {
                    if active {
                        match handle.detach_kernel_driver(iface) {
                            Ok(_) => println!("device detached"),
                            Err(e) => println!("failed to detach device: {}", e),
                        }
                    }
                }
                Err(e) => println!("unable to check device state: {}", e),
            }

            match handle.claim_interface(iface) {
                Ok(_) => {
                    set_row_colors(&mut handle, colors);
                }
                Err(e) => println!("failed to claim interface: {}", e),
            }
        }
        Err(e) => println!("unable to open usb device: {}", e),
    }
}
