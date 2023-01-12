use core::time;
use std::{env, thread};

use dotenv::dotenv;

use bme280::{Measurements, BME280};
use ltr_559::{AlsGain, AlsIntTime, AlsMeasRate, Ltr559, SlaveAddr};
use rppal::{hal, i2c::I2c};

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

trait LineProtocol {
    fn as_line_protocol(&self, measurement: &str, hostname: &str) -> String;
}

struct RoomClimateReading {
    temperature: f64,
    humidity: f64,
    pressure: f64,
    light: f64,
}

impl LineProtocol for RoomClimateReading {
    fn as_line_protocol(&self, measurement: &str, hostname: &str) -> String {
        format!(
            "{measurement},hostname={hostname} temperature={},humidity={},pressure={},light={}",
            self.temperature, self.humidity, self.pressure, self.light
        )
    }
}

fn get_env(env_name: &str) -> String {
    env::var(env_name).expect(&format!(
        "You need to provide {env_name} as environment variable!"
    ))
}

fn write_measurement(cr: RoomClimateReading) {
    let client = reqwest::blocking::Client::new();
    let url = format!(
        "{}/api/v2/write?org=influxdata&bucket=default",
        get_env("INFLUX_URL")
    );
    let authorization_token = get_env("INFLUX_AUTH_TOKEN");

    let body_value = cr.as_line_protocol("thpl", "pizero");
    println!("{body_value}");

    let resp = client
        .post(url)
        .bearer_auth(authorization_token)
        .header("Content-Type", "text/plain")
        .body(body_value)
        .send()
        .unwrap();

    println!("Statuscode: {}", resp.status());
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    loop {
        let thp = bme_measurements();
        let light = ltr_measurements();

        let room_reading = RoomClimateReading {
            temperature: thp.temperature.into(),
            humidity: thp.humidity.into(),
            pressure: thp.pressure.into(),
            light: light.into(),
        };

        write_measurement(room_reading);
        thread::sleep(time::Duration::from_secs(30));
    }
}
