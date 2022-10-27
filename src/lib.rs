//! [![crates.io](https://img.shields.io/crates/v/device-register)](https://crates.io/crates/device-register) [![documentation](https://docs.rs/device-register/badge.svg)](https://docs.rs/device-register)
//!
//! A buffer that persists between boot.
//! Inspired by [panic-persist](https://github.com/jamesmunns/panic-persist)
//!
//! A region in RAM is reseved for this buffer.
//! Your linker script should make sure the start and end of the buffer are outside of other sections
//!
//! ## Usage
//!
//! ### Linker script
//! You need to create a new reserved section for the buffer and make sure it's
//! outside of other sections to avoid zero initializations.
//!
//! #### Example
//! `memory.x` file before modification:
//!
//! ``` ignore
//! MEMORY
//! {
//!   /* NOTE K = KiBi = 1024 bytes */
//!   FLASH : ORIGIN  = 0x00000000, LENGTH = 512K
//!   RAM : ORIGIN    = 0x20000000, LENGTH = 64K
//! }
//! ```
//! `memory.x` file after modification to hold a 1K region:
//! ``` ignore
//! MEMORY
//! {
//!   /* NOTE K = KiBi = 1024 bytes */
//!   FLASH : ORIGIN  = 0x00000000, LENGTH = 512K
//!   RAM : ORIGIN    = 0x20000000, LENGTH = 63K
//!   PERSISTANT_BUFF: ORIGIN = 0x2000FC00, LENGTH = 1K
//! }
//!
//! _persistant_buff_start = ORIGIN(PERSISTANT_BUFF);
//! _persistant_buff_end   = ORIGIN(PERSISTANT_BUFF) + LENGTH(PERSISTANT_BUFF);
//! ```
//!
//! ### Program
//!
//! ```ignore
//! #![no_std]
//!
//! #[entry]
//! fn main() -> ! {
//!     let buff = unsafe { persistant_buff::get() };
//!     buff[0]++;
//!     info!("Value is now {}", buff[0]);
//! }
//! ```
#![no_std]
#![no_main]
#![deny(missing_docs)]

/// Get the persistant buff
pub unsafe fn get() -> &'static mut [u8] {
    extern "C" {
        static mut _persistant_buff_start: u8;
        static mut _persistant_buff_end: u8;
    }

    let start = &mut _persistant_buff_start as *mut u8;
    let end = &mut _persistant_buff_end as *mut u8;
    let len = end as usize - start as usize;

    let slice = core::slice::from_raw_parts_mut(start, len);
    slice
}
