# CCS811 library for Raspberry

As I am learning rust and want to use it for embedded programming I created this library. It is inspired by Marten Penning's
library for the CCS811 written in C. If you see issues with my rust code, have some tips for improvement or feature wishes
feel free to create an issue on this repository and help me to improve :).

This library depends on the rppal crate for the I2C communication with the chip. May in future this can be abstracted to
make this library work on other platforms as well.

### Wiring

|CCS811|Raspberry||
|------|---------|---|
|VCC|Pin 1 (3.3V)||
|GND|Pin 6 (GND)||
|SCL|Pin 5 (SCL & GPIO 3)||
|SDA|Pin 3 (SDA & GPIO 2)||
|WAK|Pin 11 (GPIO 17)|Optional / Can be changed|

### How to use

```rust
use rppal::i2c::I2c;
use rppal::gpio::Gpio;
use std::thread::sleep;
use std::time::Duration;
use ccs811;

fn main() {
    let i2c = I2c::with_bus(1).expect("Couldn't start i2c. Is the interface enabled?");
    let wake_pin = Gpio::new().expect("Can not init gpio")
                            .get(17).expect("Could not attach to wake pin");
    wake_pin.into_output().set_low();

    let mut ccs811 = ccs811::new(i2c, None);

    match ccs811.begin() {
        Ok(()) => match ccs811.start(ccs811::MODE::Sec1) {
            Ok(()) => (),
            Err(error) => panic!("Could not start: {}", error)
        },
        Err(error) => panic!("Could not init the chip: {}", error)
    }

    println!("Chip Bootloader Version: {:x?}", ccs811.bootloader_version().unwrap());
    println!("Chip Hardware Version: {:x?}", ccs811.hardware_version().unwrap());
    println!("Chip Application Version: {:x?}", ccs811.application_version().unwrap());

    sleep(Duration::from_secs(5));

    let mut i = 0;
    loop {
        match ccs811.read() {
            Ok(data) => {
                println!("{}mins => t_voc: {}, e_co2: {}, raw: {:x?}", i, data.t_voc, data.e_co2, data.raw);
            },
            Err(error) => println!("Could not read data: {}", error)
        };

        i += 1;
        sleep(Duration::from_secs(60))
    }
}
``` 

### How to flash new firmware

Most chips out there got the version 1.0.0 or 1.1.0. Right now where I created this readme there is 2.0.0 and 2.1.0 out there.
Those versions made the results of the chip for me way more reliable and stable. Before I had raising values without reasons
which stopped after flashing the new firmware.

You can get the current firmware from [ams.com](https://ams.com/ccs811#tab/tools). Just download the zip file and 
either take the 2.0.0 binary version for unused chips or the 2.1.0 binary for used chips (stated in the readme of the firmware).

This is an example code to flash the new firmware by creating a rust executable, wire the chip to the raspberry pi and execute it.
The following code assumes the file `CCS811_FW_App_v2-0-1.bin` to be in the same folder as the executable while execution.

```rust
use rppal::i2c::I2c;
use rppal::gpio::Gpio;
use std::fs::File;
use std::io::Read;
use ccs811;

fn main() {
    let i2c = I2c::with_bus(1).expect("Couldn't start i2c. Is the interface enabled?");
    let wake_pin = Gpio::new().expect("Can not init gpio")
        .get(17).expect("Could not attach to wake pin");
    wake_pin.into_output().set_low();

    let mut ccs811 = ccs811::new(i2c, None);

    let mut file = File::open("./CCS811_FW_App_v2-0-1.bin")
        .expect("No firmware found");
    let mut data = vec![];
    let read = file.read_to_end(&mut data)
        .expect("Could not load firmware");

    println!("Firmware has size of {} bytes", read);

    ccs811.flash(data)
        .expect("Failed to flash firmware");

    println!("Flashed :)");
}
```
