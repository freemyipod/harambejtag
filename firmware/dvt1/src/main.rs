#![no_std]
#![no_main]

// Needed for heapless pools.
#![allow(non_camel_case_types)]

// TODO: remove once we integrate core1 messaging.
#![allow(dead_code)]

mod bsp;
mod command;

use bsp::{
    entry,
    hal,
    pac,
    hal::{
        Clock,
        gpio,
        gpio::FunctionUart,
        gpio::dynpin::DynPin,
        multicore::{Multicore, Stack},
        uart,
        uart::UartPeripheral,
    },
};

use heapless::{
    String,
    pool,
    pool::{
        singleton::Pool,
        singleton::Box,
    },
};

use panic_halt as _;

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::SerialPort;

use embedded_hal::digital::v2::OutputPin;

static mut CORE1_STACK: Stack<4096> = Stack::new();

/// Message between core0/core1.
pub enum Message {
    Empty,
    DebugOut(String<128>),
}

pool!(
    MESSAGE_POOL: Message
);

type MBox = Box<MESSAGE_POOL, heapless::pool::Init>;

static mut MESSAGE_POOL_MEMORY: [u8; 4096] = [0; 4096];


/// Core1 state.
struct Core1 {
    /// JTAG buffers enable pin.
    buf_oen: DynPin,
}

impl Core1 {
    fn new(
        buf_oen: DynPin,
    ) -> Self {
        Self {
            buf_oen,
        }
    }

    fn log(&mut self, fifo: &mut hal::sio::SioFifo, s: String::<128>) {
        match MESSAGE_POOL::alloc() {
            Some(msg) => {
                let msg = msg.init(Message::DebugOut(s));
                fifo.write_blocking(&msg as &MBox as *const MBox as u32);
                core::mem::forget(msg);
            },
            _ => (),
        }
    }

    fn run(self) -> ! {
        // TODO: run core1 code
        loop {
            cortex_m::asm::wfi();
        }
    }
}


#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    unsafe {
        #[allow(static_mut_ref)]
        MESSAGE_POOL::grow(&mut MESSAGE_POOL_MEMORY);
    }

    let mut sio = hal::Sio::new(pac.SIO);
    let pins = bsp::Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);
    let _: gpio::Pin<_, gpio::FunctionPio0> = pins.jtag_tdi.into_mode();
    let _: gpio::Pin<_, gpio::FunctionPio0> = pins.jtag_tdo.into_mode();
    let _: gpio::Pin<_, gpio::FunctionPio0> = pins.jtag_tck.into_mode();
    let mut led_amber = pins.led_amber.into_push_pull_output();
    led_amber.set_high().unwrap();


    let uart_pins = (
        pins.uart_to_pod.into_mode::<FunctionUart>(),
        pins.uart_from_pod.into_mode::<FunctionUart>(),
    );
    let uart = UartPeripheral::new(pac.UART1, uart_pins, &mut pac.RESETS)
        .enable(uart::common_configs::_115200_8_N_1, clocks.peripheral_clock.freq())
        .unwrap();


    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Serial port for commands.
    let mut serial_cmd = SerialPort::new(&usb_bus);
    // Serial port for UART bridge.
    let mut serial_uart = SerialPort::new(&usb_bus);

    // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x2138))
        .manufacturer("Freemyipod")
        .product("HarambeJTAG")
        .serial_number("E1Q001")
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();

    // Start core1.
    let c1 = Core1::new(pins.buf_oen.into());
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    let _test = core1.spawn(unsafe { &mut CORE1_STACK.mem }, move || {
        c1.run();
    });


    // Run core0.
    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut said_hello = false;
    loop {
        // Print a welcome message on startup (but not too early or Linux will get confused).
        if !said_hello && timer.get_counter() >= 2_000_000 {
            said_hello = true;
            let _ = serial_cmd.write(b"Hello, World!\r\n");
        }

        // Check for new USB data
        if usb_dev.poll(&mut [&mut serial_cmd, &mut serial_uart]) {
            let mut buf = [0u8; 64];
            match serial_cmd.read(&mut buf) {
                Err(_e) => (),
                Ok(0) => (),
                Ok(_count) => {
                    // TODO: handle command
                }
            }

            // The UART TX FIFO is 32 bytes. Don't attempt to read more than that. This should be
            // enough backpressure to not cause USB timeouts.
            let mut buf = [0u8; 32];
            if uart.uart_is_writable() {
                match serial_uart.read(&mut buf) {
                    // Do nothing
                    Err(_e) => (),
                    // Do nothing
                    Ok(0) => (),
                    Ok(count) => {
                        // buf is small enough that we can probably afford to block here.
                        // TODO: do the math on this
                        uart.write_full_blocking(&buf[..count]);
                    },
                }
            }
        }

        // Check for new UART data.
        let mut buf = [0u8; 32];
        match uart.read_raw(&mut buf) {
            Err(_e) => (),
            Ok(0) => (),
            Ok(count) => {
                let mut wr_ptr = &buf[..count];
                while !wr_ptr.is_empty() {
                    match serial_uart.write(&buf[..count]) {
                        Ok(len) => wr_ptr = &wr_ptr[len..],
                        Err(_) => break,
                    }
                }
            },
        }

        // Check for new messages from core1.
        match sio.fifo.read() {
            Some(_) => {
                // TODO: parse messages from core1
            },
            _ => (),
        }
    }
}
