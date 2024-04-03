use crate::{Color, Image};
use embassy_stm32::gpio::*;
use embassy_stm32::peripherals::*;
use embassy_time::Ticker;
use embassy_time::Timer;

pub struct Matrix<'a> {
    sb: Output<'a, PC5>,
    lat: Output<'a, PC4>,
    rst: Output<'a, PC3>,
    sck: Output<'a, PB1>,
    sda: Output<'a, PA4>,
    rows: [Output<'a, AnyPin>; 8],
}

impl Matrix<'_> {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        pa2: PA2,
        pa3: PA3,
        pa4: PA4,
        pa5: PA5,
        pa6: PA6,
        pa7: PA7,
        pa15: PA15,
        pb0: PB0,
        pb1: PB1,
        pb2: PB2,
        pc3: PC3,
        pc4: PC4,
        pc5: PC5,
    ) -> Self {
        let sb = Output::new(pc5, Level::High, Speed::VeryHigh);
        let lat = Output::new(pc4, Level::High, Speed::VeryHigh);
        let rst = Output::new(pc3, Level::Low, Speed::VeryHigh);
        let sck = Output::new(pb1, Level::Low, Speed::VeryHigh);
        let sda = Output::new(pa4, Level::Low, Speed::VeryHigh);
        let rows = [
            Output::new(pb2, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pa15, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pa2, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pa7, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pa6, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pa5, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pb0, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pa3, Level::Low, Speed::VeryHigh).degrade(),
        ];
        let mut matrix = Matrix {
            sb,
            lat,
            rst,
            sck,
            sda,
            rows,
        };
        Timer::after_millis(100).await;
        matrix.rst.set_high();
        matrix.init_bank0();
        matrix
    }

    fn pulse_sck(&mut self) {
        self.sck.set_low();
        self.sck.set_high();
        self.sck.set_low();
    }

    fn pulse_lat(&mut self) {
        self.lat.set_high();
        self.lat.set_low();
        self.lat.set_high();
    }

    fn send_byte(&mut self, pixel: u8) {
        self.sb.set_high();
        for i in (0..8).rev() {
            if (pixel & (1 << i)) != 0 {
                self.sda.set_high();
            } else {
                self.sda.set_low();
            }
            self.pulse_sck();
        }
    }

    pub fn send_row(&mut self, row: usize, pixels: &[Color]) {
        let prev_row = if row == 0 {
            self.rows.len() - 1
        } else {
            row - 1
        };
        self.rows[prev_row].set_low();
        for pixel in pixels.iter().rev() {
            let gamma_corrected = pixel.gamma_correct();
            self.send_byte(gamma_corrected.b);
            self.send_byte(gamma_corrected.g);
            self.send_byte(gamma_corrected.r);
        }
        self.pulse_lat();
        self.rows[row].set_high();
    }

    fn init_bank0(&mut self) {
        self.sb.set_low();
        self.sda.set_high();
        for _ in 0..144 {
            self.pulse_sck();
        }
        self.pulse_lat();
        self.sb.set_high();
    }

    pub async fn display_image(&mut self, image: &Image, ticker: &mut Ticker) {
        for row in 0..8 {
            self.send_row(row, image.row(row));
            ticker.next().await;
        }
    }
}
