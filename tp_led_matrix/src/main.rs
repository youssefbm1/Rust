// #![no_std]
// #![no_main]

// use cortex_m_rt::entry;
// use defmt_rtt as _;
// use embassy_stm32 as _;
// use embassy_stm32::rcc::*;
// use embassy_stm32::Config;
// use panic_probe as _;
// use tp_led_matrix::Image::{BLUE, GREEN, RED};
// use tp_led_matrix::{Color, Image, Matrix};

// #[panic_handler]
// fn panic_handler(_panic_info: &core::panic::PanicInfo) -> ! {
//     loop {}
// }

// #[entry]
// fn main() -> ! {
//     defmt::info!("Hello, world!");
//     panic!("The program stopped");
//  }

// #[entry]
// fn main() -> ! {
//     let mut config = Config::default();
//     config.rcc.pll = Some(Pll {
//         source: PllSource::HSI,
//         prediv: PllPreDiv::DIV1,
//         mul: PllMul::MUL10,
//         divp: None,
//         divq: None,
//         divr: Some(PllRDiv::DIV2),
//     });

//     let p = embassy_stm32::init(config);

//     let mut matrix = Matrix::new(
//         p.PA2, p.PA3, p.PA4, p.PA5, p.PA6, p.PA7, p.PA15, p.PB0, p.PB1, p.PB2, p.PC3, p.PC4, p.PC5,
//     );

//     // Create an image with a gradient of blue
//     let mut image = Image::new_solid(Color::from_rgb(0, 0, 0));
//     for y in 0..8 {
//         for x in 0..8 {
//             let blue = (x * 32) as u8;
//             image.set_color(x, y, Rgb565::new(0, 0, blue));
//         }
//     }

//     // Display the image in a loop
//     loop {
//         for y in 0..8 {
//             matrix.await.send_row(y, image.row(y));
//             Delay.delay_ms(1);
//         }
//     }
// }

// #[entry]
// fn main() -> ! {
//     defmt::info!("defmt correctly initialized");
//     let mut config = Config::default();
//     config.rcc.mux = ClockSrc::PLL1_R;
//     config.rcc.hsi = true;
//     config.rcc.pll = Some(Pll {
//         source: PllSource::HSI,
//         prediv: PllPreDiv::DIV1,
//         mul: PllMul::MUL10,
//         divp: None,
//         divq: None,
//         divr: Some(PllRDiv::DIV2),
//     });

//     let image = Image::gradient(BLUE);
//     let p = embassy_stm32::init(config);
//     let mut matrix = Matrix::new(p.PA2, p.PA3, p.PA4, p.PA5, p.PA6, p.PA7, p.PA15, p.PB0, p.PB1, p.PB2, p.PC3, p.PC4, p.PC5,);
//     loop {
//         matrix.display_image(&image);
//     }

//     panic!("Everything configured");
// }

#![no_std]
#![no_main]
// #![feature(type_alias_impl_trait)]

use defmt::unwrap;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32 as _;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::PB14;
use embassy_stm32::rcc::*;
use embassy_stm32::Config;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Ticker, Timer};
use panic_probe as _;
use tp_led_matrix::{Color, Image, Matrix};

static IMAGE: Mutex<ThreadModeRawMutex, Image> = Mutex::new(Image::new_solid(Color::BLUE));

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let mut config = Config::default();
    config.rcc.mux = ClockSrc::PLL1_R;
    config.rcc.hsi = true;
    config.rcc.pll = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL10,
        divp: None,
        divq: None,
        divr: Some(PllRDiv::DIV2),
    });
    let p = embassy_stm32::init(config);

    let matrix = Matrix::new(
        p.PA2, p.PA3, p.PA4, p.PA5, p.PA6, p.PA7, p.PA15, p.PB0, p.PB1, p.PB2, p.PC3, p.PC4, p.PC5,
    )
    .await;

    spawner.spawn(blinker(p.PB14)).unwrap();
    spawner.spawn(display(matrix)).unwrap();
}

#[embassy_executor::task]
async fn blinker(pb14: PB14) {
    let mut green_led = Output::new(pb14, Level::Low, Speed::VeryHigh);
    loop {
        for _ in 0..3 {
            green_led.set_high();
            Timer::after_millis(100).await;
            green_led.set_low();
            Timer::after_millis(100).await;
        }
        Timer::after_millis(3000).await;
    }
}

#[embassy_executor::task]
async fn display(mut matrix: Matrix<'static>) {
    let mut ticker = Ticker::every(Duration::from_hz(640));
    loop {
        let image = IMAGE.try_lock().unwrap();
        matrix.display_image(&image, &mut ticker).await;
        ticker.next().await;
    }
}
