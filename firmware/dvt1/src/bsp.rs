pub use rp2040_hal as hal;

extern crate cortex_m_rt;
pub use hal::entry;

#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_IS25LP080;

pub use hal::pac;

hal::bsp_pins!(
    Gpio0 { name: jtag_rtck },
    Gpio1 { name: jtag_tdo },
    Gpio2 { name: jtag_ntrst },
    Gpio3 { name: jtag_tdi },
    Gpio4 { name: jtag_tck },
    Gpio5 { name: jtag_tms },

    Gpio6 { name: buf_oen },

    Gpio8 { name: uart_to_pod },
    Gpio9 { name: uart_from_pod },

    Gpio13 { name: led_green },
    Gpio14 { name: led_amber }
    Gpio15 { name: led_red },

    Gpio24 { name: ext_24 },
    Gpio25 { name: ext_25 },
    Gpio26 { name: ext_26 },
    Gpio27 { name: ext_27 },
    Gpio28 { name: ext_28 },
    Gpio29 { name: ext_29 },
);

pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;
