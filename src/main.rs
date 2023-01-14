use core::time;
use std::{env, thread};

use chrono::Local;
use dotenv::dotenv;

use bme280::{Measurements, BME280};
use ltr_559::{AlsGain, AlsIntTime, AlsMeasRate, Ltr559, SlaveAddr};
use rppal::{hal, i2c::I2c};

use influxdb_line_protocol::{DataPoint, FieldValue};

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

fn get_env(env_name: &str) -> String {
    env::var(env_name).expect(&format!(
        "You need to provide {env_name} as environment variable!"
    ))
}

fn send(datapoint_str: &String) -> reqwest::blocking::Response {
    let client = reqwest::blocking::Client::new();
    let url = format!(
        "{}/api/v2/write?org=influxdata&bucket=default",
        get_env("INFLUX_URL")
    );
    let authorization_token = get_env("INFLUX_AUTH_TOKEN");

    let resp = client
        .post(url)
        .bearer_auth(authorization_token)
        .header("Content-Type", "text/plain")
        .body(datapoint_str.clone())
        .send()
        .unwrap();

    resp
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    let hostname = sys_info::hostname().unwrap();
    loop {
        let now = Local::now();
        let thp = bme_measurements();
        let light = ltr_measurements();

        let datapoint = DataPoint {
            measurement: "thpl",
            tag_set: vec![("hostname", hostname.as_str())],
            field_set: vec![
                ("temperature", FieldValue::Float(thp.temperature.into())),
                ("pressure", FieldValue::Float(thp.pressure.into())),
                ("humidity", FieldValue::Float(thp.humidity.into())),
                ("light", FieldValue::Float(light.into())),
            ],
            timestamp: None,
        };
        let dp_string = datapoint.into_string().unwrap();
        let resp = send(&dp_string);

        println!("[{}] {} - {}", now, resp.status(), dp_string);

        thread::sleep(time::Duration::from_secs(30));
    }
}
