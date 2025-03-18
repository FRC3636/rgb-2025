use std::io::{self, Read, Write};
use ws2812_spi::hosted::Ws2812;

#[derive(Debug)]
pub struct SpiBus {
    spi: spidev::Spidev,
}
impl SpiBus {
    pub fn open(bus: &str) -> io::Result<Self> {
        let mut spi = spidev::Spidev::open(bus)?;
        let options = spidev::SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(3_800_000)
            .mode(spidev::SpiModeFlags::SPI_MODE_0)
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
        self.spi
            .read_exact(words)
            .map_err(|_| embedded_hal::spi::ErrorKind::Other)?;
        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        let mut transfer = spidev::SpidevTransfer::write(words);
        self.spi
            .transfer(&mut transfer)
            .map_err(|_| embedded_hal::spi::ErrorKind::Other)?;
        _ = self.spi.flush();

        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        dbg!();
        assert!(read.len() == write.len());
        let mut transfer = spidev::SpidevTransfer::read_write(write, read);
        self.spi
            .transfer(&mut transfer)
            .map_err(|_| embedded_hal::spi::ErrorKind::Other)?;

        Ok(())
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        dbg!();
        let mut rx_buf = vec![0; words.len()];

        let mut transfer = spidev::SpidevTransfer::read_write(words, &mut rx_buf);
        self.spi
            .transfer(&mut transfer)
            .map_err(|_| embedded_hal::spi::ErrorKind::Other)?;

        words.copy_from_slice(&rx_buf);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn gpio_10() -> std::io::Result<Ws2812<SpiBus>> {
    let dev = SpiBus::open("/dev/spidev0.0")?;
    Ok(Ws2812::new(dev))
}
pub fn gpio_18() -> std::io::Result<Ws2812<SpiBus>> {
    let dev = SpiBus::open("/dev/spidev1.0")?;
    Ok(Ws2812::new(dev))
}
