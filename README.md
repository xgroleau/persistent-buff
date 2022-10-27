# persistent-buff

[![crates.io](https://img.shields.io/crates/v/device-register)](https://crates.io/crates/device-register) [![documentation](https://docs.rs/device-register/badge.svg)](https://docs.rs/device-register)

A buffer that persists between boot.
Inspired by [panic-persist](https://github.com/jamesmunns/panic-persist)

A region in RAM is reseved for this buffer.
Your linker script should make sure the start and end of the buffer are outside of other sections

### Usage

#### Linker script
You need to create a new reserved section for the buffer and make sure it's
outside of other sections to avoid zero initializations.

##### Example
`memory.x` file before modification:

``` ignore
MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  FLASH : ORIGIN = 0x00000000, LENGTH = 1024K
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}
```rust
`memory.x` file after modification to hold a 1K region:
``` ignore
MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  FLASH : ORIGIN = 0x00000000, LENGTH = 1024K
  RAM : ORIGIN = 0x20000000, LENGTH = 128K - 1K
  PERSISTENT_BUFF: ORIGIN = ORIGIN(RAM) + LENGTH(RAM), LENGTH = 1K
}
_persistent_buff_start = ORIGIN(PERSISTENT_BUFF);
_persistent_buff_end   = ORIGIN(PERSISTENT_BUFF) + LENGTH(PERSISTENT_BUFF);
```

#### Program

```rust
#![no_std]

#[entry]
fn main() -> ! {
   let mut pbuff = persistent_buff::PersistentBuff::take_managed().unwrap();

   // Trivial way to initialize is to fill it with 0
   let buff = pbuff.validate(|b| b.fill(0));

   buff[0] = (buff[0] % 255) + 1;
   info!("Value is now {}", buff[0]);
}
```
