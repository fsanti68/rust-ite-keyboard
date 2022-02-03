use std::collections::HashMap;
use std::time::Duration;
use palette::rgb::Rgb;

use libusb::{Device, DeviceHandle, DeviceList, Direction, DeviceDescriptor};

use crate::device_error::DeviceError;

static SETUP_COMMAND: [u8; 8] = [0x08, 0x02, 0x33, 0x00, 0x24, 0x00, 0x00, 0x00];

static REQUEST_TYPE: u8 = 0x21;
static REQUEST: u8 = 0x9;
static VALUE: u16 = 0x300;


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
    let result: DeviceResult<Device> = Err(DeviceError::DeviceNotFound);
    for mut device in devices.iter() {
        match device.device_descriptor() {
            Ok(device_desc) => {
                match device.config_descriptor(0u8) {
                    Ok(_) => {
                        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
                            println!("{:?}", device_desc);

                            let endpoint = get_out_endpoint(&mut device, device_desc);
                            match endpoint {
                                Some(number) => println!("endpoint {}", number),
                                _ => println!("endpoint not found"),
                            }


                            return Ok(device);
                        }
                    },
                    Err(_) => println!("unable to get config descriptor"),
                }
            }
            Err(_) => println!("unable to get device descriptor"),
        }
    }
    return result;
}

fn get_out_endpoint(device: &mut Device, device_desc: DeviceDescriptor) -> Option<u8> {
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                println!("Interface {}", 
                    interface_desc.description_string_index().unwrap_or(0)
                );

                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    // print_endpoint(&endpoint_desc);
                    match endpoint_desc.direction() {
                        Direction::Out => {
                            return Some(endpoint_desc.number());
                        },
                        _ => {},
                    }
                }
            }
        }
    }
    None
}

pub fn setup_mode(k: &mut Device, mode: &str) {
    let timeout = Duration::from_secs(2);
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
                    match handle.write_control(
                        REQUEST_TYPE, REQUEST, VALUE,
                        iface as u16, // config index
                        &SETUP_COMMAND,
                        timeout
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("could not init: {}", e);
                            return;
                        }
                    }

                    // set mode
                    let ctrl_buff: Option<&[u8; 8]> = LIGHT_MODES.get(mode);
                    match ctrl_buff {
                        Some(b) => {
                            // send change mode command
                            println!("Setting mode {}", mode);
                            match handle.write_control(REQUEST_TYPE, REQUEST, VALUE, iface as u16, b, timeout) {
                                Ok(_siz) => println!("Ok"),
                                Err(e) => println!("Could not set mode: {}", e),
                            }
                        }
                        None => println!("Unknown mode '{}'", mode),
                    }
                }
                Err(e) => println!("failed to claim interface: {}", e),
            }
        }
        Err(e) => println!("unable to open usb device: {}", e),
    }
}

fn set_row_colors(handle: &mut DeviceHandle, iface: u8, endpoint: u8, colors: &[[u32; 21]; 6]) {
    for r in 0..colors.len() {
        let mut msg: [u8; 64] = [0; 64];
        for c in 0..20 {
            let rgb = Rgb::from_u32(&colors[r][c]);
            msg[c + 0] /* blue */ = 255 * rgb.blue;
            msg[c + 21] /* green */ = 255 * rgb.green;
            msg[c + 42] /* red */ = 255 * rgb.red;
        }

        // send setup for row "r"
        let command: [u8; 8] = [0x16, 0, r as u8, 0, 0, 0, 0, 0];
        //println!("setup row: {:?}", command);
        let r_wrote = handle.write_control(
            REQUEST_TYPE, REQUEST, VALUE,
            iface as u16,
            &command,
            Duration::from_millis(500),
        );

        match r_wrote {
            Ok(d_siz) => {
                print!("row {:2}: ctrl ok ({} bytes)", r, d_siz);
                // send colors array for row
                match handle.write_bulk(endpoint, &msg, Duration::from_millis(500)) {
                    Ok(siz) => println!(", data ok ({} bytes)", siz),
                    Err(e) => println!(", data write failure \"{}\"", e),
                }
            }
            Err(e) => println!("Could not set color: {}", e),
        }
    }
}

pub fn set_color(device: &mut Device, colors: &[[u32; 21]; 6]) {
    //println!("Device found: {:?}", device);
    match device.open() {
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
                    // send setup command
                    match handle.write_control(
                        REQUEST_TYPE, REQUEST, VALUE,
                        iface as u16, // config index
                        &SETUP_COMMAND,
                        Duration::from_secs(2),
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("could not init: {}", e);
                            return;
                        }
                    }

                    match device.config_descriptor(0u8) {
                        Ok(_) => {
                            let device_desc = device.device_descriptor().unwrap();
                            match get_out_endpoint(device, device_desc) {
                                Some(endpoint) => set_row_colors(&mut handle, iface, endpoint, colors),
                                _ => println!("no valid out endpoint"),
                            }
                        },
                        Err(_) => {},
                    }
                },
                Err(e) => println!("failed to claim interface: {}", e),
            }
        }
        Err(e) => println!("unable to open usb device: {}", e),
    }
}
