#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use stm32f1xx_hal::{prelude::*, stm32, stm32::interrupt};
use stm32f1xx_hal::usb::{UsbBus, Peripheral};
use usb_device::test_class::TestClass;

static mut INTERRUPT_COUNTER: u32 = 0;

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    assert!(clocks.usbclk_valid());

    let gpioa = dp.GPIOA.split(&mut rcc.apb2);

    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: gpioa.pa12,
    };
    let usb_bus = UsbBus::new(usb);

    let mut test = TestClass::new(&usb_bus);

    let mut usb_dev = { test.make_device(&usb_bus) };

    unsafe {
        NVIC::unmask(stm32::Interrupt::USB_HP_CAN_TX);
        NVIC::unmask(stm32::Interrupt::USB_LP_CAN_RX0);
    }

    loop {
        cortex_m::asm::wfi();

        unsafe {
            if INTERRUPT_COUNTER > 10 {
                panic!("too many interrupt calls")
            }
        }

        if usb_dev.poll(&mut [&mut test]) {
            test.poll();
        }

        unsafe {
            NVIC::unmask(stm32::Interrupt::USB_HP_CAN_TX);
            NVIC::unmask(stm32::Interrupt::USB_LP_CAN_RX0);
        }
    }
}

#[interrupt]
fn USB_HP_CAN_TX() {
    // avoid continuously re-entering this interrupt handler
    NVIC::mask(stm32::Interrupt::USB_HP_CAN_TX);

    unsafe { INTERRUPT_COUNTER += 1; }
}

#[interrupt]
fn USB_LP_CAN_RX0() {
    // avoid continuously re-entering this interrupt handler
    NVIC::mask(stm32::Interrupt::USB_LP_CAN_RX0);

    unsafe { INTERRUPT_COUNTER += 1; }
}
