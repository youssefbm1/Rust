#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32 as _;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::{DMA1_CH5, PB14, PB6, PB7, USART1};
use embassy_stm32::usart::Uart;
use embassy_stm32::Config;
use embassy_stm32::{bind_interrupts, rcc::*, usart};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Ticker, Timer};
use panic_probe as _;
use tp_led_matrix::{Color, Image, Matrix};

static mut IMAGE: Mutex<ThreadModeRawMutex, Image> = Mutex::new(Image::new_solid(Color::BLACK));

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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

    // let mut ticker = Ticker::every(Duration::from_secs(1));
    // loop {
    //     unsafe {
    //         IMAGE = Mutex::new(Image::gradient(Color::BLUE));
    //     }
    //     ticker.next().await;
    //     unsafe {
    //         IMAGE = Mutex::new(Image::gradient(Color::RED));
    //     }
    //     ticker.next().await;
    //     unsafe {
    //         IMAGE = Mutex::new(Image::gradient(Color::GREEN));
    //     }
    //     ticker.next().await;
    // }
    spawner
        .spawn(serial_receiver(p.USART1, p.PB6, p.PB7, p.DMA1_CH5))
        .unwrap();
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
        unsafe {
            for i in 0..8 {
                let local_buffer: &mut [Color; 8] = &mut [Color::BLACK; 8];
                {
                    let image_lock = IMAGE.lock().await;
                    local_buffer.copy_from_slice(&image_lock.row(i)[..8]);
                }
                matrix.send_row(i, local_buffer);
                ticker.next().await;
            }
        };
    }
}

#[embassy_executor::task]
async fn serial_receiver(usart1: USART1, pb6: PB6, pb7: PB7, dma1_ch5: DMA1_CH5) {
    bind_interrupts!(struct Irqs{
        USART1 => usart::InterruptHandler<USART1>;
});
    let mut config = usart::Config::default();
    config.baudrate = 38400;
    let mut serial = Uart::new(usart1, pb7, pb6, Irqs, NoDma, dma1_ch5, config).unwrap();
    let mut buffer = [0_u8; 192];
    loop {
    rm .git/index.lock   let mut c = 0;
        serial.read(core::slice::from_mut(&mut c)).await.unwrap();
        if c != 0xff {
            continue;
        }
        let mut start = 0;
        'receive: loop {
            serial.read(&mut buffer[start..]).await.unwrap();
            for pos in (start..192).rev() {
                if buffer[pos] == 0xff {
                    buffer.rotate_left(pos+1);
                    start = 192 - (pos + 1);
                    continue 'receive;
                }
            }
            break;
        }
        let mut image = unsafe {IMAGE.lock().await};
        *image.as_mut() = buffer;
        drop(image);
    }
}