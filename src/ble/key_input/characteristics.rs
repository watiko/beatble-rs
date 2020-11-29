use std::collections::HashSet;
use std::sync::{atomic, Arc};
use std::thread;

use bluster::{
    gatt::{
        characteristic::{Characteristic, Properties},
        descriptor::Descriptor,
        event::Event,
    },
    SdpShortUuid,
};
use futures::channel::mpsc::channel;
use futures::StreamExt;
use log::{debug, info};
use tokio::time::Duration;
use uuid::Uuid;

const CHARACTERISTIC_UUID: u16 = 0xFF01;
const SLEEP_DURATION: Duration = Duration::from_millis(500);

pub fn create_key_input_characteristic(descriptors: HashSet<Descriptor>) -> Characteristic {
    debug!("create_key_input_characteristic");

    let (sender, receiver) = channel(1);

    let characteristic_handler = async {
        debug!("create_key_input_characteristic: handler spawned");
        let notifying = Arc::new(atomic::AtomicBool::new(false));
        let mut rx = receiver;
        while let Some(event) = rx.next().await {
            match event {
                Event::NotifySubscribe(notify_subscribe) => {
                    debug!("notify request to UUID({}) received", CHARACTERISTIC_UUID);
                    let notifying = Arc::clone(&notifying);
                    notifying.store(true, atomic::Ordering::Relaxed);

                    thread::spawn(move || loop {
                        debug!("send notify from UUID({})", CHARACTERISTIC_UUID);
                        if !(&notifying).load(atomic::Ordering::Relaxed) {
                            break;
                        };

                        notify_subscribe
                            .clone()
                            .notification
                            .try_send(vec![0x01, 0x02])
                            .unwrap();

                        thread::sleep(SLEEP_DURATION);
                    });
                }
                Event::NotifyUnsubscribe => {
                    debug!(
                        "unsubscribe request to UUID({}) received",
                        CHARACTERISTIC_UUID
                    );
                    notifying.store(false, atomic::Ordering::Relaxed);
                }
                _ => {
                    info!(
                        "unimplemented event detected on key input characteristics: {:?}",
                        event
                    );
                }
            }
        }
    };

    tokio::spawn(characteristic_handler);

    Characteristic::new(
        Uuid::from_sdp_short_uuid(CHARACTERISTIC_UUID),
        Properties::new(None, None, Some(sender), None),
        None,
        descriptors,
    )
}
