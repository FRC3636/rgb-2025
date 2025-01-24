use std::{
    io::{self, Write},
    path::Path,
};
use ws2812_spi::Ws2812;

#[derive(Debug)]
pub struct SpiBus {
    spi: spidev::Spidev,
}
impl SpiBus {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let mut spi = spidev::Spidev::open(path)?;
        let options = spidev::SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(3_800_000) // 125 MHz
            .build();
        spi.configure(&options)?;
        Ok(Self { spi })
    }
}
impl embedded_hal::spi::ErrorType for SpiBus {
    type Error = embedded_hal::spi::ErrorKind;
}

impl embedded_hal::spi::SpiBus for SpiBus {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        let mut transfer = spidev::SpidevTransfer::read(words);
        self.spi
            .transfer(&mut transfer)
            .map_err(|_| embedded_hal::spi::ErrorKind::Other)?;
        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        let mut transfer = spidev::SpidevTransfer::write(words);
        self.spi
            .transfer(&mut transfer)
            .map_err(|_| embedded_hal::spi::ErrorKind::Other)?;
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        assert_eq!(read.len(), write.len());
        let mut transfer = spidev::SpidevTransfer::read_write(write, read);
        self.spi
            .transfer(&mut transfer)
            .map_err(|_| embedded_hal::spi::ErrorKind::Other)?;
        Ok(())
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        let mut rx_buf = vec![0; words.len()];
        let mut transfer = spidev::SpidevTransfer::read_write(words, &mut rx_buf);
        self.spi
            .transfer(&mut transfer)
            .map_err(|_| embedded_hal::spi::ErrorKind::Other)?;
        words.copy_from_slice(&rx_buf);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.spi
            .flush()
            .map_err(|_| embedded_hal::spi::ErrorKind::Other)?;

        Ok(())
    }
}

macro_rules! gpio_strip {
    ($file:expr => $fun:ident) => {
        pub fn $fun() -> std::io::Result<Ws2812<SpiBus>> {
            let dev = SpiBus::open($file)?;
            Ok(Ws2812::new(dev))
        }
    };
}
gpio_strip!("/dev/spidev0.1" => gpio_10);
gpio_strip!("/dev/spidev1.0" => gpio_18);
