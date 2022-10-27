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
//!   /* NOTE 1 K = 1 KiBi = 1024 bytes */
//!   FLASH : ORIGIN = 0x00000000, LENGTH = 1024K
//!   RAM : ORIGIN = 0x20000000, LENGTH = 128K
//! }
//! ```
//! `memory.x` file after modification to hold a 1K region:
//! ``` ignore
//! MEMORY
//! {
//!   /* NOTE 1 K = 1 KiBi = 1024 bytes */
//!   FLASH : ORIGIN = 0x00000000, LENGTH = 1024K
//!   RAM : ORIGIN = 0x20000000, LENGTH = 128K - 1K
//!   PERSISTENT_BUFF: ORIGIN = ORIGIN(RAM) + LENGTH(RAM), LENGTH = 1K
//! }
//! _persistent_buff_start = ORIGIN(PERSISTENT_BUFF);
//! _persistent_buff_end   = ORIGIN(PERSISTENT_BUFF) + LENGTH(PERSISTENT_BUFF);
//! ```
//!
//! ### Program
//!
//! ```ignore
//! #![no_std]
//!
//! #[entry]
//! fn main() -> ! {
//!    let mut pbuff = persistent_buff::PersistentBuff::take_managed().unwrap();
//!
//!    // Trivial way to initialize is to fill it with 0
//!    let buff = pbuff.validate(|b| b.fill(0));
//!
//!    buff[0] = (buff[0] % 255) + 1;
//!    info!("Value is now {}", buff[0]);
//! }
//! ```
#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::sync::atomic::{AtomicBool, Ordering};

const MAGIC_NUMBER: u32 = 0x42069F;
static mut PERSISTENT_BUFF_TAKEN: AtomicBool = AtomicBool::new(false);

/// Strut to request the persistent buff and manage it `safely`.
/// When acquiring the buffer you need to validate it and init it to a known sate
pub struct PersistentBuff {
    magic: *mut u32,
    buff: &'static mut [u8],
}

impl PersistentBuff {
    /// Take a managed version fo the persistent buff.
    /// Allow to check if the buffer is valid or not before usage
    /// Note that vs the [Self::take] function, you will lose sii
    pub fn take_managed() -> Option<Self> {
        Self::take().map(|b| Self {
            magic: b.as_mut_ptr().cast::<u32>(),
            buff: &mut b[core::mem::size_of::<u32>()..],
        })
    }

    /// Steal a managed version for the persistent buff without check
    /// See [Self::take_managed]
    pub unsafe fn steal_managed() -> Self {
        let b = Self::steal();
        Self {
            magic: b.as_mut_ptr().cast::<u32>(),
            buff: &mut b[core::mem::size_of::<u32>()..],
        }
    }

    /// Get the raw persistent buff
    pub fn take() -> Option<&'static mut [u8]> {
        unsafe {
            if PERSISTENT_BUFF_TAKEN.swap(true, Ordering::Relaxed) {
                None
            } else {
                Some(Self::steal())
            }
        }
    }

    /// Steal the raw persistent buff.
    /// Ignore if it was already taken or not.
    pub unsafe fn steal() -> &'static mut [u8] {
        PERSISTENT_BUFF_TAKEN.store(true, Ordering::SeqCst);
        extern "C" {
            static mut _persistent_buff_start: u8;
            static mut _persistent_buff_end: u8;
        }
        let start = &mut _persistent_buff_start as *mut u8;
        let end = &mut _persistent_buff_end as *mut u8;
        let len = end as usize - start as usize;

        let slice = core::slice::from_raw_parts_mut(start, len);
        slice
    }

    /// Mark the persistent buffer with valid data in it
    fn mark(&mut self) {
        unsafe {
            *self.magic = MAGIC_NUMBER;
        }
    }

    /// Verify if the persistent buffer has valid data in it
    fn check(&self) -> bool {
        unsafe { *self.magic == MAGIC_NUMBER }
    }

    /// Check if the buffer is valid, if not call the provided closure
    /// Then mark the buffer as valid and initialize it to a known state.
    /// It's to make sure the data in it is always "valid" and not garbage after a powerloss.
    pub fn validate<F>(&mut self, f: F) -> &mut [u8]
    where
        F: FnOnce(&mut [u8]),
    {
        if !self.check() {
            f(self.buff)
        }
        self.mark();
        self.buff
    }
}
