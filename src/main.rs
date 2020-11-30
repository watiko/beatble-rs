use bluster::Peripheral;
use log::info;

use crate::input::create_input_handler;

use self::ble::create_key_input;

mod ble;
mod input;

const ADVERTISING_NAME: &str = "IIDX Entry model";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    info!("Preparing input handler");
    let key_input = create_input_handler()?;

    info!("Preparing peripheral");
    let peripheral = Peripheral::new().await?;
    peripheral.add_service(&create_key_input(key_input))?;

    while !peripheral.is_powered().await? {}
    info!("Peripheral powered on");

    peripheral.register_gatt().await?;
    peripheral.start_advertising(ADVERTISING_NAME, &[]).await?;

    while !peripheral.is_advertising().await? {}
    info!("Peripheral started advertising {}", ADVERTISING_NAME);

    while peripheral.is_advertising().await? {}
    info!("Peripheral stopped advertising {}", ADVERTISING_NAME);

    Ok(())
}
