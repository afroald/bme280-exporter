use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use bme280_rs::{Bme280, Configuration, Oversampling};
use clap::Parser;
use linux_embedded_hal::{Delay, I2cdev};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};
use tokio::sync::Mutex;
use tracing::{info, Level};

#[derive(Parser)]
#[clap(name = "bme280-exporter", version, author)]
struct Cli {
    i2c_device_path: PathBuf,

    #[arg(long, default_value_t = Ipv4Addr::new(127, 0, 0, 1))]
    host: Ipv4Addr,

    #[arg(long, default_value_t = 3000)]
    port: u16,
}

struct AppState {
    prometheus: PrometheusHandle,
    bme280: Mutex<Bme280<I2cdev, Delay>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let cli = Cli::parse();

    let prometheus = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to setup prometheus metrics");

    metrics::register_gauge!("temperature");
    metrics::describe_gauge!("temperature", "Temperature in °C");
    metrics::register_gauge!("pressure");
    metrics::describe_gauge!("pressure", "Air pressure in mPa");
    metrics::register_gauge!("humidity");
    metrics::describe_gauge!("humidity", "Relative humidity in %");

    info!(
        i2c_device_path = cli.i2c_device_path.display().to_string(),
        "connecting to i2c bus",
    );
    let i2c_bus = I2cdev::new(cli.i2c_device_path).expect("failed to setup i2c bus");
    let mut bme280 = Bme280::new_with_address(i2c_bus, 0x77, Delay);

    info!("initializing bme280 sensor");
    bme280.init().expect("failed to setup bme280 sensor");

    info!("configuring bme280 sensor");
    bme280
        .set_sampling_configuration(
            Configuration::default()
                .with_filter(bme280_rs::Filter::Filter4)
                .with_temperature_oversampling(Oversampling::Oversample8)
                .with_pressure_oversampling(Oversampling::Oversample8)
                .with_humidity_oversampling(Oversampling::Oversample8)
                .with_sensor_mode(bme280_rs::SensorMode::Forced),
        )
        .expect("failed to configure bme280 sensor");

    let app_state = AppState {
        prometheus,
        bme280: Mutex::new(bme280),
    };

    let app = Router::new()
        .route("/metrics", get(metrics))
        .with_state(Arc::new(app_state));

    axum::Server::bind(&SocketAddr::new(IpAddr::V4(cli.host), cli.port))
        .serve(app.into_make_service())
        .await
        .expect("http server failed");
}

async fn metrics(State(app_state): State<Arc<AppState>>) -> Result<String, AppError> {
    let mut bme280 = app_state.bme280.lock().await;

    bme280.take_forced_measurement()?;
    let (temperature, pressure, humidity) = bme280.read_sample()?;

    if let Some(temperature) = temperature {
        metrics::gauge!("temperature", f64::from(temperature));
    }

    if let Some(pressure) = pressure {
        metrics::gauge!("pressure", f64::from(pressure));
    }

    if let Some(humidity) = humidity {
        metrics::gauge!("humidity", f64::from(humidity));
    }

    Ok(app_state.prometheus.render())
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("kapot: {:?}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
