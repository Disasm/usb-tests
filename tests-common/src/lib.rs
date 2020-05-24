mod device;
mod stboot;

use crate::device::open_device;
use std::sync::Mutex;
use lazy_static::lazy_static;
use libusb::Context;
use usb_switch_common::{ResetStatus, Boot0Status};
use std::thread::sleep;
use std::time::Duration;
use crate::stboot::Bootloader;
use std::path::PathBuf;
use std::env;
use std::process::Command;

lazy_static! {
    static ref DEVICE_IDS: Mutex<Vec<u16>> = Mutex::new(Vec::new());
}

fn get_device_id(bootloader: &mut Bootloader) -> Option<u16> {
    bootloader.init().ok()?;
    bootloader.cmd_get().ok()?;
    bootloader.get_device_id().ok()
}

pub fn select_chip(device_id: u16) {
    let ctx = Context::new().expect("create libusb context");
    let mut dev = open_device(&ctx).expect("open device");

    let mut selection = dev.get_selection().expect("get current selection");

    let mut device_ids: Vec<u16> = DEVICE_IDS.lock().unwrap().clone();
    if device_ids.is_empty() {
        for channel in 0..256 {
            selection.set_reset(ResetStatus::Asserted).set_boot0(Boot0Status::Deasserted);
            selection.set_power_enabled(false).set_usb_enabled(false);
            dev.select(selection).expect("poweroff failed");

            sleep(Duration::from_millis(500));

            selection.set_channel(channel as u8);
            if dev.select(selection).is_err() {
                break;
            }

            selection.set_power_enabled(true);
            dev.select(selection).expect("poweron failed");

            selection.set_boot0(Boot0Status::Asserted).set_reset(ResetStatus::Asserted);
            dev.select(selection).expect("reset failed");
            selection.set_reset(ResetStatus::Deasserted);
            dev.select(selection).expect("reset failed");

            sleep(Duration::from_millis(500));

            let serial = serialport::open(&dev.serial_path).expect("serial open");
            let mut bootloader = Bootloader::new(serial);

            let device_id = get_device_id(&mut bootloader).unwrap_or(0);
            device_ids.push(device_id);
        }

        DEVICE_IDS.lock().unwrap().extend_from_slice(&device_ids);
        eprintln!("device ids: {:?}", device_ids);
    }

    for (channel, id) in device_ids.iter().enumerate() {
        if *id == device_id {
            selection.set_reset(ResetStatus::Asserted).set_boot0(Boot0Status::Deasserted);
            selection.set_power_enabled(false).set_usb_enabled(false);
            selection.set_channel(channel as u8);
            dev.select(selection).expect("select failed");
            return;
        }
    }
    panic!("Can't find target with device id 0x{:x}", device_id);
}

pub fn flash_firmware(example: &str, features: &[&str]) {
    let ctx = Context::new().expect("create libusb context");
    let mut dev = open_device(&ctx).expect("open device");

    let mut selection = dev.get_selection().expect("get current selection");

    selection.set_power_enabled(true).set_reset(ResetStatus::Asserted);
    dev.select(selection).expect("poweron failed");
    selection.set_boot0(Boot0Status::Asserted);
    dev.select(selection).expect("boot0 update failed");
    selection.set_reset(ResetStatus::Deasserted);
    dev.select(selection).expect("reset failed");

    sleep(Duration::from_millis(500));

    let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let firmware_dir = project_dir.join("fw");

    let mut command = Command::new("cargo");
    command.args(&["run", "--release", "--example", example]);
    if !features.is_empty() {
        let features = features.join(",");
        command.args(&["--features", &features]);
    }
    println!("Running command {:?}", command);

    let status = command.current_dir(firmware_dir).status().expect("failed to execute process");
    if !status.success() {
        panic!("Cargo command failed");
    }
}

pub fn shutdown() {
    let ctx = Context::new().expect("create libusb context");
    let mut dev = open_device(&ctx).expect("open device");

    let mut selection = dev.get_selection().expect("get current selection");

    selection.set_reset(ResetStatus::Asserted).set_boot0(Boot0Status::Deasserted);
    selection.set_power_enabled(false).set_usb_enabled(false);
    dev.select(selection).expect("poweroff failed");
}

pub fn run_and_connect() {
    let ctx = Context::new().expect("create libusb context");
    let mut dev = open_device(&ctx).expect("open device");

    let mut selection = dev.get_selection().expect("get current selection");

    selection.set_reset(ResetStatus::Asserted).set_boot0(Boot0Status::Deasserted);
    selection.set_power_enabled(false).set_usb_enabled(false);
    dev.select(selection).expect("poweroff failed");

    sleep(Duration::from_millis(500));

    selection.set_power_enabled(true);
    dev.select(selection).expect("poweron failed");
    selection.set_reset(ResetStatus::Deasserted);
    dev.select(selection).expect("reset failed");
    selection.set_usb_enabled(true);
    dev.select(selection).expect("usb enable failed");
}

pub fn run_usb_device_tests() {
    let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let usb_device_dir = project_dir.join("../usb-device");

    let status = Command::new("cargo")
        .arg("test")
        .current_dir(usb_device_dir)
        .status()
        .expect("failed to execute process");
    if !status.success() {
        panic!("usb-device tests failed");
    }
}
