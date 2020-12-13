use std::sync::Arc;

use bluster::Peripheral;
use clap::{value_t, App, Arg};
use crossbeam::atomic::AtomicCell;
use log::{debug, info};

use crate::input::{create_input_handler, KeyInput};

use self::ble::create_key_input;

mod ble;
mod input;

const ADVERTISING_NAME: &str = "IIDX Entry model";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let matches = App::new("beatble")
        .arg(
            Arg::with_name("input")
                .help("input device")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("sleep_duration")
                .help("DURATION should be number of milliseconds")
                .long("sleep-duration")
                .value_name("DURATION")
                .default_value("8"), // 1000 / 120
        )
        .get_matches();

    let input = value_t!(matches, "input", String).unwrap();
    let sleep_duration = value_t!(matches, "sleep_duration", u64).unwrap_or(8);
    debug!("input: {}", input);
    debug!("sleep_duration: {}", sleep_duration);

    let sleep_duration = tokio::time::Duration::from_millis(sleep_duration);

    info!("Preparing input handler");
    let key_input = create_input_handler()?;

    run_peripheral(key_input, sleep_duration).await
}

async fn run_peripheral(
    key_input: Arc<AtomicCell<KeyInput>>,
    sleep_duration: tokio::time::Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Preparing peripheral");
    let peripheral = Peripheral::new().await?;
    peripheral.add_service(&create_key_input(key_input, sleep_duration))?;

    while !peripheral.is_powered().await? {}
    info!("Peripheral powered on");

    peripheral.register_gatt().await?;
    peripheral.start_advertising(ADVERTISING_NAME, &[]).await?;

    while !peripheral.is_advertising().await? {}
    info!("Peripheral started advertising {}", ADVERTISING_NAME);

    while peripheral.is_advertising().await? {
        tokio::time::delay_for(tokio::time::Duration::from_secs(1)).await;
    }
    info!("Peripheral stopped advertising {}", ADVERTISING_NAME);

    Ok(())
}
