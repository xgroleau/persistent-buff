# persistent-buff

[![crates.io](https://img.shields.io/crates/v/persistent-buff)](https://crates.io/crates/persistent-buff) [![documentation](https://docs.rs/persistent-buff/badge.svg)](https://docs.rs/persistent-buff)

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

### License
Licensed under either of
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
