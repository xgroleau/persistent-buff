//! [![crates.io](https://img.shields.io/crates/v/persistent-buff)](https://crates.io/crates/persistent-buff) [![documentation](https://docs.rs/persistent-buff/badge.svg)](https://docs.rs/persistent-buff)
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
//! ```text
//! MEMORY
//! {
//!   /* NOTE 1 K = 1 KiBi = 1024 bytes */
//!   FLASH : ORIGIN = 0x00000000, LENGTH = 1024K
//!   RAM : ORIGIN = 0x20000000, LENGTH = 128K
//! }
//! ```
//!
//! `memory.x` file after modification to hold a 1K region:
//! ```text
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
//!
//! ## License
//! Licensed under either of
//! - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
//!   <http://www.apache.org/licenses/LICENSE-2.0>)
//!
//! - MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
//!
//! at your option.
//!
//! ## Contribution
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::sync::atomic::{AtomicBool, Ordering};

const MAGIC_NUMBER: u32 = 0xFAB42069;
static mut PERSISTENT_BUFF_TAKEN: AtomicBool = AtomicBool::new(false);

/// Strut to request the persistent buff and manage it somewhat "safely".
/// When acquiring the buffer you need to validate/init it to a known sate.
pub struct PersistentBuff {
    magic: *mut u32,
    buff: &'static mut [u8],
}

impl PersistentBuff {
    /// Take a managed version fo the persistent buff.
    /// Allows to check if the buffer is valid or not before usage.
    /// Note that compared to the [Self::take] function, you will lose some bytes for storage of the marker.
    pub fn take_managed() -> Option<Self> {
        Self::take_raw().map(|b| Self {
            magic: b.as_mut_ptr().cast::<u32>(),
            buff: &mut b[core::mem::size_of::<u32>()..],
        })
    }

    /// Steal a managed version for the persistent buff without check.
    /// See [Self::take_managed]
    ///
    /// # Safety
    /// Calling this function could allow to have two mutable reference to the same buffer.
    /// Make sure to only have one reference at a time to avoid multiple mutable reference.
    pub unsafe fn steal_managed() -> Self {
        let b = Self::steal();
        Self {
            magic: b.as_mut_ptr().cast::<u32>(),
            buff: &mut b[core::mem::size_of::<u32>()..],
        }
    }

    /// Get the raw persistent slice.
    pub fn take_raw() -> Option<&'static mut [u8]> {
        unsafe {
            if PERSISTENT_BUFF_TAKEN.swap(true, Ordering::Relaxed) {
                None
            } else {
                Some(Self::steal())
            }
        }
    }

    /// Steal the raw persistent slice.
    /// Ignore if it was already taken or not.
    ///
    /// # Safety
    /// Calling this function could allow to have two mutable reference to the same buffer.
    /// Make sure to only have one reference at a time to avoid multiple mutable reference.
    pub unsafe fn steal() -> &'static mut [u8] {
        PERSISTENT_BUFF_TAKEN.store(true, Ordering::SeqCst);
        extern "C" {
            static mut _persistent_buff_start: u8;
            static mut _persistent_buff_end: u8;
        }
        let start = &mut _persistent_buff_start as *mut u8;
        let end = &mut _persistent_buff_end as *mut u8;
        let len = end as usize - start as usize;

        core::slice::from_raw_parts_mut(start, len)
    }

    /// Mark the persistent buffer with valid data in it.
    fn mark(&mut self) {
        unsafe {
            self.magic.write_unaligned(MAGIC_NUMBER);
        }
    }

    /// Unmark the persistent buffer with valid data in it.
    fn unmark(&mut self) {
        unsafe {
            self.magic.write_unaligned(0);
        }
    }

    /// Verify if the persistent buffer has valid data in it.
    pub fn valid(&self) -> bool {
        unsafe { self.magic.read_unaligned() == MAGIC_NUMBER }
    }

    /// Take the static internal buffer from the managed buff if valid
    pub fn take(self) -> Option<&'static mut [u8]> {
        if self.valid() {
            return Some(self.buff);
        } else {
            return None;
        }
    }

    /// Force to reset the buffer to a known state via the closure and mark as valid for next boot then
    /// takes the static buff from the managed buff
    pub fn take_reset<F>(mut self, f: F) -> &'static mut [u8]
    where
        F: FnOnce(&mut [u8]),
    {
        f(self.buff);
        self.mark();
        self.buff
    }

    /// Check if the buffer is valid, if not call the provided closure.
    /// Then mark the buffer as valid and initialize it to a known state.
    /// This is to make sure the data in it is always "valid" and not garbage after a powerloss.
    /// Then the static buff is taken from the managed buff
    pub fn take_validate<F>(mut self, f: F) -> &'static mut [u8]
    where
        F: FnOnce(&mut [u8]),
    {
        if !self.valid() {
            f(self.buff)
        }
        self.mark();
        self.buff
    }

    /// Get the buffer if the data is valid, if not, return None
    pub fn get(&mut self) -> Option<&mut [u8]> {
        if self.valid() {
            return Some(self.buff);
        } else {
            return None;
        }
    }

    /// Force reset the buffer to a known state via the closure, mark as valid and return the buffer
    pub fn reset<F>(&mut self, f: F) -> &mut [u8]
    where
        F: FnOnce(&mut [u8]),
    {
        f(self.buff);
        self.mark();
        self.buff
    }

    /// Check if the buffer is valid, if not call the provided closure.
    /// Then mark the buffer as valid.
    /// This is to make sure the data in it is always "valid" and not garbage after a powerloss.
    pub fn validate<F>(&mut self, f: F) -> &mut [u8]
    where
        F: FnOnce(&mut [u8]),
    {
        if !self.valid() {
            f(self.buff)
        }
        self.mark();
        self.buff
    }

    /// Mark the buffer as invalid
    pub fn invalidate(&mut self) {
        self.unmark();
    }
}
