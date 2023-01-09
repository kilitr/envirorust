use core::time;
use std::thread;

use dotenv::dotenv;

use bme280::{Measurements, BME280};
use ltr_559::{AlsGain, AlsIntTime, AlsMeasRate, Ltr559, SlaveAddr};
use rppal::{hal, i2c::I2c};

use futures::prelude::*;
use influxdb2::models::DataPoint;
use influxdb2::Client;

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

struct RoomClimateReading {
    temperature: f64,
    humidity: f64,
    pressure: f64,
    light: f64,
}

async fn write_measurement(cr: RoomClimateReading) -> Result<(), Box<dyn std::error::Error>> {
    let host = std::env::var("INFLUXDB_HOST").unwrap();
    let org = std::env::var("INFLUXDB_ORG").unwrap();
    let token = std::env::var("INFLUXDB_TOKEN").unwrap();
    let bucket = std::env::var("INFLUXDB_BUCKET").unwrap();
    let client = Client::new(host, org, token);

    let points = vec![DataPoint::builder("room")
        .tag("room", "Kilian")
        .field("temperature", cr.temperature)
        .field("humidity", cr.humidity)
        .field("pressure", cr.pressure)
        .field("light", cr.light)
        .build()?];
    client.write(bucket.as_str(), stream::iter(points)).await?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    loop {
        let thp = bme_measurements();
        let light = ltr_measurements();

        println!(
            "{}°C {}% {}pascal {}lux",
            thp.temperature, thp.humidity, thp.pressure, light
        );

        let room_reading = RoomClimateReading {
            temperature: thp.temperature.into(),
            humidity: thp.humidity.into(),
            pressure: thp.pressure.into(),
            light: light.into(),
        };

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(write_measurement(room_reading))
            .unwrap();

        thread::sleep(time::Duration::from_secs(30));
    }
}
