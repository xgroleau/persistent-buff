#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _;


#[cortex_m_rt::entry]
fn main() -> ! {
    let buff = unsafe { persistant_buff::get() };
    buff[0] += 1;
    info!("Value is now {}", buff[0]);
}
