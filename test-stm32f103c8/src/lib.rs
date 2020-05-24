#![cfg_attr(not(test), no_std)]

#[cfg(test)]
use tests_common::*;

#[test]
fn dummy() {
    select_chip(0x410);
}

#[test]
fn test_class() {
    select_chip(0x410);
    flash_firmware("test_class", &[]);
    run_and_connect();
    run_usb_device_tests();
    shutdown();
}
