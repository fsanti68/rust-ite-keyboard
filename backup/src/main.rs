#![feature(rustc_private)]
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate libusb;

use std::env;

mod device_error;
mod device_setup;

fn get_range(s: &str, max: usize) -> Vec<usize> {
    let mut r: Vec<usize> = Vec::new();
    let lines: Vec<&str> = s.split(",").collect();
    for line in lines {
        let x: Vec<&str> = line.split("-").collect();
        if x.len() == 1 {
            if "all".eq(x[0]) {
                for j in 0..(max + 1) {
                    r.push(j);
                }
            } else {
                let i_ini: usize = x[0].parse().unwrap();
                r.push(i_ini);
            }
        } else {
            let i_ini: usize = x[0].parse().unwrap();
            let i_end: usize = x[1].parse().unwrap();
            for j in i_ini..(i_end + 1) {
                r.push(j);
            }
        }
    }
    return r;
}

fn usage(args: &Vec<String>, root_warning: bool) {
    let prg_name = args.get(0).unwrap();
    let spaces = " ".repeat(prg_name.len());
    if root_warning {
        println!("This program requires root privileges\n");
    }
    println!("usage: {} [cmd] [args]", prg_name);
    println!("where 'cmd' and 'args' can be:");
    println!("  mode [off | fade | wave | dots | rainbow | explosion | snake | raindrops]");
    println!("  color [rows range] [columns range] [color name]");
    if !root_warning {
        println!("\n\nColumn and row ranges can be expressed as:");
        println!(" - single value:         3          row #3");
        println!(" - list of values:       0,2,4      rows 0, 2 and 4");
        println!(" - range of values:      5-17       columns from 5 to 17");
        println!(" - mixed list and range: 0-2,17,18  columns 0, 1, 2, 17 and 18");
        println!(" - all possible values:  all        rows from 0 to 5 and columns from 0 to 20");
        println!("\n\nSome examples:");
        println!(
            "  # {} mode snake                       starts snake mode",
            prg_name
        );
        println!(
            "  # {} color 0 all yellow               yellow on row #0 keys",
            prg_name
        );
        println!(
            "  # {} color 0,5 all red 2-4 all white  red on rows 0 and 5 and white on rows 2 to 4",
            prg_name
        );
        println!(
            "  # {} color 1,4-5 0-1,18-19 blue       blue for the first and",
            prg_name
        );
        println!(
            "    {}                                  last 2 keys of rows 1, 4 and 5",
            spaces
        );
    }
}

fn list_devices() {
    let mut ctx = libusb::Context::new().unwrap();
    let devices = ctx.devices().unwrap();
    device_setup::list_devices(devices);
}

fn set_mode(mode: &str) {
    let mut ctx = libusb::Context::new().unwrap();
    let devices = ctx.devices().unwrap();
    let keyboard = device_setup::get_keyboard(devices, 0x048d, 0xce00);

    match keyboard {
        Ok(mut k) => device_setup::setup_mode(&mut k, mode),
        Err(e) => println!("{}", e),
    }
}

fn set_color(colors: &[[&str; 21]; 6]) {
    let mut ctx = libusb::Context::new().unwrap();
    let devices = ctx.devices().unwrap();
    let keyboard = device_setup::get_keyboard(devices, 0x048d, 0xce00);

    match keyboard {
        Ok(mut k) => device_setup::set_color(&mut k, colors),
        Err(e) => println!("{}", e),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let uid = unsafe { libc::getuid() };

    if args.len() < 2 || uid != 0 {
        usage(&args, uid != 0);
        std::process::exit(1);
    }

    for mut i in 0..args.len() {
        let cmd = &args[i];
        if "-h".eq(cmd) || "--help".eq(cmd) {
            usage(&args, false);
        } else if "list".eq(cmd) {
            list_devices();
        } else if "mode".eq(cmd) {
            let mode = &(args[i + 1]).to_lowercase();
            println!("set mode to {}", mode);
            set_mode(mode);
        } else if "color".eq(cmd) {
            let mut colors: [[&str; 21]; 6] = [[&"black"; 21]; 6];
            while i < args.len() - 2 {
                let rows = get_range(&args[i + 1].to_lowercase(), 5);
                let cols = get_range(&args[i + 2].to_lowercase(), 20);
                let color = &args[i + 3];
                i += 3;
                for row in rows {
                    for col in cols.clone() {
                        colors[row][col] = &color;
                    }
                }
                set_color(&colors);
            }
        }
    }
}
