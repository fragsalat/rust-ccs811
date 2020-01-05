use rppal::i2c::I2c;
use rppal::gpio::Pin;
use crate::chip::CCS811;

mod chip;
mod constants;

pub use crate::constants::Ccs811Mode as MODE;

/// Creates a new instance of the chip. Be aware that in my experiences the wake pin resulted in wrong data.
/// This probably is caused due to the short heating period after the awakening. To save energy I would
/// set the Sec60 mode and leave it awake.
///
/// # Examples
///
/// ```
/// use rppal::i2c::I2c;
/// use rppal::gpio::Gpio;
///
/// let i2c = I2c::with_bus(1).expect("Couldn't start i2c. Is the interface enabled?");
/// let wake_pin = Gpio::new().expect("Can not init gpio")
///                    .get(17).expect("Could not attach to wake pin");
///
/// let mut ccs811 = ccs811::new(i2c, Some(wake_pin));
/// ```
pub fn new(i2c: I2c, wake: Option<Pin>) -> CCS811 {
    let chip = CCS811 {
        i2c,
        // Put wake pin into output mode if set
        wake: wake.map(|pin| pin.into_output())
    };

    return chip;
}