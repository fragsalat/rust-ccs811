use rppal::i2c::I2c;
use rppal::gpio::{OutputPin};
use std::thread::sleep;
use std::cmp::min;
use super::constants::{*};
use std::result::Result::Err;

/// Bytes are calculated by taking the value without fraction and put it's 7 bits to the first byte.
/// The fraction is multiplied by 512 as described in the CCS811 specs. To ensure
/// The value can not be higher than 127 but humidity and temperature, this function is used for, will never
/// exceed this.
fn float_to_bytes(value: f32) -> [u8; 2] {
    let base = value.floor();
    // We only have 9 bits. 512 are already 10. So we ensure with min() that max 511 is used for fraction
    let fraction = min(((value - base) * 512.0 - 1.0) as u16, 511);
    // Take 7 bits of base and 1 bit of fraction
    let hi = ((base as u8 & 0b1111111) << 1) | ((&fraction & 0b100000000) >> 8) as u8;
    // Take 8 bits of fraction (the missing one is in the high byte
    let lo = (&fraction & 0xFF) as u8;

    [hi, lo]
}

pub struct Ccs811Data {
    pub t_voc: u16,
    pub e_co2: u16,
    pub raw: Vec<u8>
}

pub struct CCS811 {
    pub i2c: I2c,
    pub wake: Option<OutputPin>
}

impl CCS811 {

    fn reset(&mut self) -> Result<(), String> {
        self.i2c.block_write(CCS811_SW_RESET, &[0x11,0xE5,0x72,0x8A])
            .map_err(|error| format!("Couldn't write to I2C: {}", error))?;

        sleep(CCS811_WAIT_AFTER_RESET_US);

        Ok(())
    }

    fn app_start(&mut self) -> Result<(), String> {
        self.i2c.write(&[CCS811_APP_START])
            .map_err(|error| format!("Could not set App start: {}", error))?;

        sleep(CCS811_WAIT_AFTER_APPSTART_US);

        Ok(())
    }

    fn erase_app(&mut self) -> Result<(), String> {
        self.i2c.block_write(CCS811_APP_ERASE, &[0xE7, 0xA7, 0xE6, 0x09])
            .map_err(|error| format!("Could not erase app: {}", error))?;

        sleep(CCS811_WAIT_AFTER_APPERASE_MS);

        Ok(())
    }

    fn check_hw_id(&mut self) -> Result<(), String> {
        let hw_id = self.i2c.smbus_read_byte(CCS811_HW_ID)
            .map_err(|error| format!("Couldn't read HWID: {}", error))?;

        if hw_id != 0x81 {
            return Err(format!("HWID of chip is not 0x81 but {:x?}", hw_id));
        }

        Ok(())
    }

    fn check_status(&mut self, expected: u8) -> Result<(), String> {
        let status = self.i2c.smbus_read_byte(CCS811_STATUS)
            .map_err(|error| format!("Could not read chip status: {}", error))?;

        if (status & expected) == 0 {
            return Err(format!("Chip status is not {:#010b} but {:#010b}", expected, status));
        }

        Ok(())
    }

    fn awake(&mut self) {
        if let Some(pin) = &mut self.wake {
            pin.set_low();
            sleep(CCS811_WAIT_AFTER_WAKE_US);
        }
    }

    fn sleep(&mut self) {
        if let Some(pin) = &mut self.wake {
            pin.set_high();
        }
    }

    /// Initialize CCS811 chip with i2c bus
    /// Sequence: set i2c slave -> Wake to low -> reset chip -> check hardware id -> start chip -> check chip status -> Wake to high -> ready
    ///
    /// # Examples
    ///
    /// ```
    /// let mut ccs811 = ccs811::new(i2c, None);
    ///
    /// match ccs811.begin() {
    ///   Ok(()) => println!("Chip is ready"),
    ///   Err(error) => panic!("Could not init the chip: {}", error)
    /// }
    /// ```
    pub fn begin(&mut self) -> Result<(), String> {
        self.i2c.set_slave_address(CCS811_SLAVEADDR_0)
            .map_err(|error| format!("Could not set slave addr: {}", error))?;

        self.awake();

        self.reset()
            .and(self.check_hw_id())
            .and(self.app_start())
            .and(self.check_status(CCS811_STATUS_APP_MODE & CCS811_STATUS_APP_VERIFY))?;

        self.sleep();

        Ok(())
    }

    /// Put CCS811 chip into target mode. Be aware that the first sampled data will be available after
    /// the period of time the mode takes. For instance it will take at least 60 seconds data will be
    /// first available in the Sec60 mode. For the Sec10 mode it is at least 10 seconds etc.
    /// Also be aware that the documentation of the chip mentions to change the chip mode to a lower
    /// sampling rate like Sec1 to Sec60, the mode should be set to Idle for at least 10 minutes before
    /// the setting the new mode.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut ccs811 = ccs811::new(i2c, None);
    ///
    /// match ccs811.begin() {
    ///   Ok(()) => match ccs811.start(ccs811::MODE::Sec1) {
    ///     Ok(()) => (),
    ///     Err(error) => panic!("Could not start: {}", error)
    ///   },
    ///   Err(error) => panic!("Could not init the chip: {}", error)
    /// }
    /// ```
    pub fn start(&mut self, mode: Ccs811Mode) -> Result<(), String> {
        self.awake();
        self.i2c.block_write(CCS811_MEAS_MODE, &[(mode as u8) << 4])
            .map_err(|error| format!("Could not set mode: {}", error))?;
        self.sleep();

        Ok(())
    }

    /// Version should be something like 0x1X
    pub fn hardware_version(&mut self) -> Result<u8, String> {
        self.i2c.smbus_read_byte(CCS811_HW_VERSION)
            .map_err(|error| format!("Could not read hardware version: {}", error))
    }

    /// Something like 0x10 0x0
    pub fn bootloader_version(&mut self) -> Result<[u8; 2], String> {
        let mut buffer = [0; 2];
        self.i2c.block_read(CCS811_FW_BOOT_VERSION, &mut buffer)
            .map_err(|error| format!("Could not read boot loader version: {}", error))?;

        Ok(buffer)
    }

    /// Something like 0x10 0x0 or higher. You can flash a newer firmware (2.0.0) using the flash method
    /// and a firmware binary. See examples for more details
    pub fn application_version(&mut self) -> Result<[u8; 2], String> {
        let mut buffer = [0; 2];
        self.i2c.block_read(CCS811_FW_APP_VERSION, &mut buffer)
            .map_err(|error| format!("Could not read application version: {}", error))?;

        Ok(buffer)
    }

    /// Get the currently used baseline
    pub fn get_baseline(&mut self) -> Result<u16, String> {
        self.i2c.smbus_read_word(CCS811_BASELINE)
            .map_err(|error| format!("Could not read baseline: {}", error))
    }

    /// The CCS811 chip has an automatic baseline correction based on a 24 hour interval but you still
    /// can set the baseline manually if you want.
    pub fn set_baseline(&mut self, baseline: u16) -> Result<(), String> {
        self.i2c.smbus_write_word(CCS811_BASELINE, baseline)
            .map_err(|error| format!("Could not set baseline: {}", error))
    }

    /// Set environmental data measured by external sensors to the chip to include those in
    /// calculations. E.g. humidity 48.5% and 23.3°C
    ///
    /// # Examples
    ///
    /// ```
    /// match ccs811.set_env_data(48.5, 23.3) {
    ///   Ok(()) => println!("Updated environmental data on chip"),
    ///   Err(error) => panic!("Failed to set environmental data on chip because {}", error)
    /// }
    /// ```
    pub fn set_env_data(&mut self, humidity: f32, temperature: f32) -> Result<(), String> {
        let data = [
            float_to_bytes(humidity),
            float_to_bytes(temperature)
        ].concat();

        self.i2c.block_write(CCS811_ENV_DATA, &data)
            .map_err(|error| format!("Could npt write env data: {}", error))?;

        Ok(())
    }

    /// Read last sampled eCO2, tVOC and the corresponding status, error and raw data from the
    /// chip register
    ///
    /// # Examples
    ///
    /// ```
    /// match ccs811.read() {
    ///   Ok(data) => {
    ///     println!("t_voc: {}, e_co2: {}, raw: {:x?}", data.t_voc, data.e_co2, data.raw);
    ///   },
    ///   Err(error) => println!("Could not read data: {}", error)
    /// };
    /// ```
    pub fn read(&mut self) -> Result<Ccs811Data, String> {
        let mut buffer = [0; 8];
        self.awake();

        self.i2c.block_read(CCS811_ALG_RESULT_DATA, &mut buffer)
            .map_err(|error| format!("Could not read chip data: {}", error))?;

        self.sleep();

        if buffer[5] != 0 {
            return Err(format!("Some error while reading data {:x?}", buffer[5]));
        }

        let data = Ccs811Data {
            e_co2: buffer[0] as u16 * 256 + buffer[1] as u16,
            t_voc: buffer[2] as u16 * 256 + buffer[3] as u16,
            raw: buffer.to_vec()
        };

        if data.t_voc > 1187 || data.e_co2 > 8192 {
            return Err(format!("The data is above max {}ppb, {}ppm", data.t_voc, data.e_co2));
        }

        Ok(data)
    }

    /// Flash another firmware to the CCS811 chip. The firmware can be found in the world wide web in
    /// form of an binary file which must be read and passed as byte array to this function.
    /// If flashing fails the chip still got a working boot loader which makes it possible to write
    /// another firmware to the chip and fix the issue.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use std::io::Read;
    ///
    /// let mut ccs811 = ccs811::new(i2c, None);
    ///
    /// let mut file = File::open("./CCS811_FW_App_v2-0-1.bin")
    ///     .expect("No firmware found");
    /// let mut data = vec![];
    /// let read = file.read_to_end(&mut data)
    ///     .expect("Could not load firmware");
    ///
    /// println!("Firmware has size of {} bytes", read);
    ///
    /// ccs811.flash(data)
    /// .expect("Failed to flash firmware");
    ///
    /// println!("Flashed :)");
    /// ```
    pub fn flash(&mut self, data: Vec<u8>) -> Result<(), String> {
        self.i2c.set_slave_address(CCS811_SLAVEADDR_0)
            .map_err(|error| format!("Could not set slave addr: {}", error))?;

        self.reset()?;
        self.check_status(CCS811_STATUS_APP_VALID)
            .map_err(|error| format!("Not valid: {}", error))?; //status!=0x00 && status!=0x10
        self.erase_app()?;
        self.check_status(CCS811_STATUS_APP_ERASE)
            .map_err(|error| format!("Not erased: {}", error))?; // status!=0x40

        let mut i = 0;
        loop {
            println!("Flashing {} of {}\r", i, data.len());
            if i >= data.len() {
                break;
            }
            let end = match i + 8 {
                v if v > data.len() => data.len(),
                v => v
            };
            self.i2c.block_write(CCS811_APP_DATA, &data[i..end])
                .map_err(|error| format!("Could not write firmware: {}", error))?;

            i += 8;
        }
        sleep(CCS811_WAIT_AFTER_APPDATA_MS);

        self.i2c.write(&[CCS811_APP_VERIFY])
            .map_err(|error| format!("Could not reset verify bit: {}", error))?;
        sleep(CCS811_WAIT_AFTER_APPVERIFY_MS);

        self.check_status(CCS811_STATUS_APP_ERASE | CCS811_STATUS_APP_VERIFY | CCS811_STATUS_APP_VALID)
            .map_err(|error| format!("Not verified: {}", error))?;

        self.reset()?;

        self.check_status(CCS811_STATUS_APP_VALID)
            .map_err(|error| format!("Unexpected status after flashing: {}", error))
    }
}



