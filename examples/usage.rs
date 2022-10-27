#![no_std]
#![no_main]

use defmt_rtt as _; // global logger
use panic_probe as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    let buff = persistant_buff::PersistantBuff::take().unwrap();
    buff[0] += 1;
    defmt::info!("Value is now {}", buff[0]);
    loop {}
}
