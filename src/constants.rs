use std::time::Duration;

pub enum Ccs811Mode {
    Idle = 0,
    Sec1 = 1,
    Sec10 = 2,
    Sec60 = 3
}

pub const CCS811_SLAVEADDR_0: u16 = 0x5A;
// pub const CCS811_SLAVEADDR_1: u16 = 0x5B;

// CCS811 registers/mailboxes, all 1 byte except when stated otherwise
pub const CCS811_STATUS          : u8 = 0x00;
pub const CCS811_MEAS_MODE       : u8 = 0x01;
pub const CCS811_ALG_RESULT_DATA : u8 = 0x02; // up to 8 bytes
// pub const CCS811_RAW_DATA        : u8 = 0x03; // 2 bytes
pub const CCS811_ENV_DATA        : u8 = 0x05; // 4 bytes
// pub const CCS811_THRESHOLDS      : u8 = 0x10; // 5 bytes
pub const CCS811_BASELINE        : u8 = 0x11; // 2 bytes
pub const CCS811_HW_ID           : u8 = 0x20;
pub const CCS811_HW_VERSION      : u8 = 0x21;
pub const CCS811_FW_BOOT_VERSION : u8 = 0x23; // 2 bytes
pub const CCS811_FW_APP_VERSION  : u8 = 0x24; // 2 bytes
// pub const CCS811_ERROR_ID        : u8 = 0xE0;
pub const CCS811_APP_ERASE       : u8 = 0xF1; // 4 bytes
pub const CCS811_APP_DATA        : u8 = 0xF2; // 9 bytes
pub const CCS811_APP_VERIFY      : u8 = 0xF3; // 0 bytes
pub const CCS811_APP_START       : u8 = 0xF4; // 0 bytes
pub const CCS811_SW_RESET        : u8 = 0xFF; // 4 bytes

pub const CCS811_STATUS_APP_MODE   : u8 = 0b10000000; // Else boot mode
pub const CCS811_STATUS_APP_ERASE  : u8 = 0b01000000; // Else no erase completed
pub const CCS811_STATUS_APP_VERIFY : u8 = 0b00100000; // Else no verify completed
pub const CCS811_STATUS_APP_VALID  : u8 = 0b00010000; // Else no valid app firmware loaded
// pub const CCS811_STATUS_DATA_READY : u8 = 0b00001000; // Else no new data samples ready
// pub const CCS811_STATUS_ERROR      : u8 = 0b00000001; // Else no error

pub const CCS811_WAIT_AFTER_RESET_US: Duration = Duration::from_micros(2000); // The CCS811 needs a wait after reset
pub const CCS811_WAIT_AFTER_APPSTART_US: Duration = Duration::from_micros(1000); // The CCS811 needs a wait after app start
pub const CCS811_WAIT_AFTER_WAKE_US: Duration = Duration::from_micros(50); // The CCS811 needs a wait after WAKE signal
pub const CCS811_WAIT_AFTER_APPERASE_MS: Duration = Duration::from_millis(500); // The CCS811 needs a wait after app erase (300ms from spec not enough)
pub const CCS811_WAIT_AFTER_APPVERIFY_MS: Duration = Duration::from_millis(70); // The CCS811 needs a wait after app verify
pub const CCS811_WAIT_AFTER_APPDATA_MS: Duration = Duration::from_millis(50); // The CCS811 needs a wait after writing app data