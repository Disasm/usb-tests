#![cfg_attr(not(test), no_std)]

#[cfg(test)]
use tests_common::*;

#[test]
fn dummy() {
    select_chip(0x410);
}
