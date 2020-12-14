use std::os::unix::io::RawFd;

use bitflags::bitflags;
use eyre::Result;
use nix::errno::Errno;
use nix::{fcntl, ioctl_read, ioctl_read_buf, unistd};
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Event {
    ButtonPressed(u8),
    ButtonReleased(u8),
    AxisChanged(u8, i16),
    Disconnected,
    Error(String),
}

#[derive(Debug, Error)]
pub enum OpenError {
    #[error("DeviceFileNotFound: {0}")]
    DeviceFileNotFound(String),
    #[error("PermissionDenied: {0}")]
    PermissionDenied(String),
    #[error("InvalidPath")]
    InvalidPath,
    #[error("Unknown: {0}")]
    Unknown(eyre::Report),
}

#[allow(dead_code)]
pub struct DeviceInfo {
    axes: u8,
    buttons: u8,
    name: String,
}

impl std::fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

bitflags! {
    struct EventType: u8 {
        const BUTTON = 0x01;
        const AXIS = 0x02;
        const INIT = 0x80;
    }
}

#[repr(C)]
struct RawEvent {
    time: u32,
    value: i16,
    typ: EventType,
    number: u8,
}

const JS_IOC_MAGIC: u8 = b'j';
const JS_IOC_TYPE_GET_AXES: u8 = 0x11;
const JS_IOC_TYPE_GET_BUTTONS: u8 = 0x12;
const JS_IOC_TYPE_GET_NAME: u8 = 0x13;

ioctl_read!(js_get_axes, JS_IOC_MAGIC, JS_IOC_TYPE_GET_AXES, u8);
ioctl_read!(js_get_buttons, JS_IOC_MAGIC, JS_IOC_TYPE_GET_BUTTONS, u8);
ioctl_read_buf!(js_get_name, JS_IOC_MAGIC, JS_IOC_TYPE_GET_NAME, u8);

impl From<RawEvent> for Option<Event> {
    #[inline]
    fn from(ev: RawEvent) -> Self {
        if ev.typ.contains(EventType::INIT) {
            // ignore init event
            return None;
        }
        match ev.typ {
            EventType::BUTTON => {
                if ev.value == 0 {
                    Some(Event::ButtonReleased(ev.number))
                } else {
                    Some(Event::ButtonPressed(ev.number))
                }
            }
            EventType::AXIS => Some(Event::AxisChanged(ev.number, ev.value)),
            _ => unreachable!(),
        }
    }
}

pub struct Device(RawFd);

impl Device {
    pub fn open(path: &str) -> Result<Self> {
        // mode is dummy
        let fd = fcntl::open(path, fcntl::OFlag::O_RDONLY, nix::sys::stat::Mode::S_IRUSR).map_err(
            |err| {
                use nix::Error;
                use OpenError::*;

                match err {
                    Error::Sys(Errno::ENOENT) => DeviceFileNotFound(path.to_string()),
                    Error::Sys(Errno::EPERM) => PermissionDenied(path.to_string()),
                    Error::InvalidPath | Error::InvalidUtf8 => InvalidPath,
                    e => Unknown(e.into()),
                }
            },
        )?;

        Ok(Device(fd))
    }

    pub fn info(&self) -> Result<DeviceInfo> {
        let mut axes = 0u8;
        let mut buttons = 0u8;
        let mut name = [0u8; 128];

        unsafe {
            js_get_axes(self.0, &mut axes)?;
            js_get_buttons(self.0, &mut buttons)?;
            js_get_name(self.0, &mut name)?;
        }

        let name = name.to_vec().into_iter().take_while(|&c| c != 0).collect();
        let name = String::from_utf8(name)?;

        Ok(DeviceInfo {
            axes,
            buttons,
            name,
        })
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unistd::close(self.0).unwrap();
    }
}

impl Iterator for Device {
    type Item = Event;

    #[inline]
    fn next(&mut self) -> Option<Event> {
        let mut buf = [0u8; 8];
        match unistd::read(self.0, &mut buf) {
            Ok(_) => {
                let raw_ev = unsafe { std::mem::transmute::<[u8; 8], RawEvent>(buf) };
                raw_ev.into()
            }
            Err(nix::Error::Sys(Errno::ENODEV)) => Some(Event::Disconnected),
            Err(e) => Some(Event::Error(format!("read error: {}", e))),
        }
    }
}
