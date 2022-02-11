use std::sync::Arc;

use bluster::Peripheral;
use clap::Parser;
use crossbeam::atomic::AtomicCell;
use eyre::Result;
use log::{debug, info};

use crate::input::{create_input_handler, KeyInput};

use self::ble::create_key_input;

mod ble;
mod input;

#[derive(Parser)]
#[clap(name = "beatble")]
struct Args {
    /// input device file path
    input: String,

    /// DURATION should be number of milliseconds
    #[clap(long, value_name = "DURATION", default_value_t = 8)] // 1000 / 120
    sleep_duration: u64,
}

const ADVERTISING_NAME: &str = "IIDX Entry model";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let input = args.input;
    let sleep_duration = args.sleep_duration;
    debug!("input: {}", input);
    debug!("sleep_duration: {}", sleep_duration);

    let sleep_duration = tokio::time::Duration::from_millis(sleep_duration);

    info!("Preparing input handler");
    let key_input = create_input_handler(&input)?;

    run_peripheral(key_input, sleep_duration).await
}

async fn run_peripheral(
    key_input: Arc<AtomicCell<KeyInput>>,
    sleep_duration: tokio::time::Duration,
) -> Result<()> {
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
