use std::collections::HashSet;
use std::sync::Arc;

use bluster::gatt::service::Service;
use crossbeam::atomic::AtomicCell;
use tokio::time::Duration;

use crate::input::KeyInput;

use self::{characteristics::create_key_input_characteristic, service::create_key_input_service};

mod characteristics;
mod service;

pub fn create_key_input(key_input: Arc<AtomicCell<KeyInput>>, sleep_duration: Duration) -> Service {
    create_key_input_service(true, {
        let mut characteristics = HashSet::new();
        characteristics.insert(create_key_input_characteristic(
            key_input,
            sleep_duration,
            HashSet::new(),
        ));
        characteristics
    })
}
