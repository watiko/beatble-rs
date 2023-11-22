use std::collections::HashSet;

use bluster::{
    gatt::{characteristic::Characteristic, service::Service},
    SdpShortUuid,
};

use super::uuid::Uuid;

const SERVICE_UUID: u16 = 0xFF00;

pub fn create_key_input_service(
    primary: bool,
    characteristics: HashSet<Characteristic>,
) -> Service {
    Service::new(
        Uuid::from_sdp_short_uuid(SERVICE_UUID),
        primary,
        characteristics,
    )
}
