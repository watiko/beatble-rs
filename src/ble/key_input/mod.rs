use std::collections::HashSet;

use bluster::gatt::service::Service;

use self::{characteristics::create_key_input_characteristic, service::create_key_input_service};

mod characteristics;
mod service;

pub fn create_key_input() -> Service {
    create_key_input_service(true, {
        let mut characteristics = HashSet::new();
        characteristics.insert(create_key_input_characteristic(HashSet::new()));
        characteristics
    })
}
