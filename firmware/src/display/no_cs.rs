use core::convert::Infallible;
use embedded_hal::digital::{ErrorType, OutputPin};

pub(super) struct NoCs;

impl OutputPin for NoCs {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl ErrorType for NoCs {
    type Error = Infallible;
}
