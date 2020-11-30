use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use bluster::gatt::service::Service;

use crate::input::KeyInput;

use self::{characteristics::create_key_input_characteristic, service::create_key_input_service};

mod characteristics;
mod service;

pub fn create_key_input(key_input: Arc<Mutex<KeyInput>>) -> Service {
    create_key_input_service(true, {
        let mut characteristics = HashSet::new();
        characteristics.insert(create_key_input_characteristic(key_input, HashSet::new()));
        characteristics
    })
}
