use bme280_rs::{Bme280, Configuration, Oversampling};
use clap::Parser;
use linux_embedded_hal::{Delay, I2cdev};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

#[derive(Parser)]
#[clap(name = "bme280-exporter", version, author)]
struct Cli {
    i2c_device_path: PathBuf,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    // println!("Hello, world!");
    // println!("i2c device: {}", cli.i2c_device_path.display());

    let i2c_bus = I2cdev::new(cli.i2c_device_path).unwrap();

    let mut bme280 = Bme280::new_with_address(i2c_bus, 0x77, Delay);
    bme280.init().unwrap();
    bme280
        .set_sampling_configuration(
            Configuration::default()
                .with_temperature_oversampling(Oversampling::Oversample8)
                .with_pressure_oversampling(Oversampling::Oversample8)
                .with_humidity_oversampling(Oversampling::Oversample8)
                .with_sensor_mode(bme280_rs::SensorMode::Normal),
        )
        .unwrap();
    println!("{:?}", bme280.chip_id());

    loop {
        println!("temperature: {:?}", bme280.read_temperature());
        println!("pressure: {:?}", bme280.read_pressure());
        println!("humidity: {:?}", bme280.read_humidity());
        sleep(Duration::from_millis(1000)).await;
    }
}
