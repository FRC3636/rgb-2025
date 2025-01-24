use rpi_embedded::spi::{Bus, Spi};
use std::io;
use ws2812_spi::hosted::Ws2812;

#[derive(Debug)]
pub struct SpiBus {
    spi: Spi,
}
impl SpiBus {
    pub fn open(bus: rpi_embedded::spi::Bus) -> io::Result<Self> {
        let spi = Spi::new(
            bus,
            rpi_embedded::spi::SlaveSelect::Ss0,
            2_000_000,
            rpi_embedded::spi::Mode::Mode1,
        )
        .map_err(|e| match e {
            rpi_embedded::spi::Error::Io(error) => error,
            _ => io::Error::new(io::ErrorKind::Other, "SPI creation error"),
        })?;

        Ok(Self { spi })
    }
}
impl embedded_hal::spi::ErrorType for SpiBus {
    type Error = embedded_hal::spi::ErrorKind;
}

impl embedded_hal::spi::SpiBus for SpiBus {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.spi
            .read(words)
            .map_err(|_| embedded_hal::spi::ErrorKind::Other)?;
        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.spi
            .write(words)
            .map_err(|_| embedded_hal::spi::ErrorKind::Other)?;
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        assert_eq!(read.len(), write.len());
        self.spi
            .transfer(read, write)
            .map_err(|_| embedded_hal::spi::ErrorKind::Other)?;
        Ok(())
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        let mut rx_buf = vec![0; words.len()];
        self.spi
            .transfer(&mut rx_buf, words)
            .map_err(|_| embedded_hal::spi::ErrorKind::Other)?;
        words.copy_from_slice(&rx_buf);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn gpio_10() -> std::io::Result<Ws2812<SpiBus>> {
    let dev = SpiBus::open(Bus::Spi0)?;
    Ok(Ws2812::new(dev))
}
pub fn gpio_18() -> std::io::Result<Ws2812<SpiBus>> {
    let dev = SpiBus::open(Bus::Spi1)?;
    Ok(Ws2812::new(dev))
}
