use core::time;
use std::thread;

use rppal::hal as hal;
use rppal::i2c::I2c;
use bme280::{BME280, Measurements};
use ltr_559::{Ltr559, SlaveAddr, AlsIntTime, AlsMeasRate, AlsGain};

fn bme_measurements() -> Measurements<rppal::i2c::Error> {
    let i2c_bus = I2c::new().expect("Error during i2c initialization");
    let mut bme280 = BME280::new_primary(i2c_bus, hal::Delay::new());
    bme280.init().expect("Error during bme280 initialization");
    bme280.measure().expect("Error during bme280 measurement")
}

fn ltr_measurements() -> f32 {
    let i2c_bus = I2c::new().expect("Error during i2c initialization");
    let mut sensor = Ltr559::new_device(i2c_bus, SlaveAddr::default());
    sensor
        .set_als_meas_rate(AlsIntTime::_50ms, AlsMeasRate::_50ms)
        .unwrap();
    sensor.set_als_contr(AlsGain::Gain4x, false, true).unwrap();
    let value = sensor.get_lux().unwrap();
    sensor.destroy();
    value
}

fn main() {
    loop {
        let thp = bme_measurements();
        let light = ltr_measurements();
        println!("{}°C {}% {}pascal {}lux", thp.temperature, thp.humidity, thp.pressure, light);
        thread::sleep(time::Duration::from_secs(1));
    }
}
