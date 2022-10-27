#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut pbuff = persistant_buff::PersistantBuff::take_managed().unwrap();

    // Verify the address
    let addr = unsafe { persistant_buff::PersistantBuff::steal() as *const [u8] as *const u8 };
    defmt::info!("Address {}", addr);

    // Trivial way to initialize it to fill it with 0
    let buff = pbuff.validate(|b| b.fill(0));

    buff[0] = (buff[0] % 255) + 1;
    defmt::info!("Value is now {}", buff[0]);
    cortex_m::asm::bkpt();
    loop {}
}
