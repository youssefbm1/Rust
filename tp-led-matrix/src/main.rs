#![no_std]
#![no_main]

use embassy_stm32::rcc::*;
use embassy_stm32::Config;

use cortex_m_rt::entry;
use embassy_stm32 as _; 
use panic_probe as _;
use defmt_rtt as _;

// #[panic_handler]
// fn panic_handler(_panic_info: &core::panic::PanicInfo) -> ! {
//     loop {}
// }

// #[entry]
// fn main() -> ! {
//     defmt::info!("Hello, world!");
//     panic!("The program stopped");
//  }

#[entry]
fn main() -> ! {
    defmt::info!("defmt correctly initialized");

    // Setup the clocks at 80MHz using HSI (by default since HSE/MSI
    // are not configured): HSI(16MHz)×10/2=80MHz. The flash wait
    // states will be configured accordingly.
    let mut config = Config::default();
    config.rcc.mux = ClockSrc::PLL1_R;
    config.rcc.hsi = true;
    config.rcc.pll = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL10,
        divp: None,
        divq: None,
        divr: Some(PllRDiv::DIV2), // 16 * 10 / 2 = 80MHz
    });
    embassy_stm32::init(config);

    panic!("Everything configured");
}