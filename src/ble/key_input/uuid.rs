use bluster::SdpShortUuid;

// avoid depending uuid crate directly
pub struct Uuid;
impl SdpShortUuid<u16> for Uuid {}
