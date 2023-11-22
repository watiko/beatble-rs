// https://www.kernel.org/doc/Documentation/input/joystick-api.txt
// https://github.com/torvalds/linux/blob/v5.10/include/uapi/linux/joystick.h

use std::os::unix::io::RawFd;

use crate::input::platform::linux::ioctl::CorrectionType;
use bitflags::bitflags;
use eyre::Result;
use nix::errno::Errno;
use nix::{fcntl, unistd};
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
    #[error("InvalidPath: {0}")]
    InvalidPath(String),
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
    #[derive(PartialEq, Eq)]
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

mod ioctl {
    use std::mem::size_of;

    use nix::errno::Errno;
    use nix::{ioctl_read, ioctl_read_buf, libc, request_code_read, request_code_write};

    #[repr(u16)]
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub enum CorrectionType {
        None = 0x00,
        Broken = 0x01,
    }

    impl Default for CorrectionType {
        fn default() -> Self {
            CorrectionType::None
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Default)]
    pub struct JsCorrection {
        pub coefficients: [i32; 8],
        pub precision: i16,
        pub typ: CorrectionType,
    }

    const JS_IOC_MAGIC: u8 = b'j';
    const JS_IOC_TYPE_GET_AXES: u8 = 0x11;
    const JS_IOC_TYPE_GET_BUTTONS: u8 = 0x12;
    const JS_IOC_TYPE_GET_NAME: u8 = 0x13;
    const JS_IOC_TYPE_SET_CORRECTION: u8 = 0x21;
    const JS_IOC_TYPE_GET_CORRECTION: u8 = 0x22;

    ioctl_read!(js_get_axes, JS_IOC_MAGIC, JS_IOC_TYPE_GET_AXES, u8);
    ioctl_read!(js_get_buttons, JS_IOC_MAGIC, JS_IOC_TYPE_GET_BUTTONS, u8);
    ioctl_read_buf!(js_get_name, JS_IOC_MAGIC, JS_IOC_TYPE_GET_NAME, u8);

    const REQ_SET_CORRECTION: libc::c_ulong = request_code_write!(
        JS_IOC_MAGIC,
        JS_IOC_TYPE_SET_CORRECTION,
        size_of::<JsCorrection>()
    );
    const REQ_GET_CORRECTION: libc::c_ulong = request_code_read!(
        JS_IOC_MAGIC,
        JS_IOC_TYPE_GET_CORRECTION,
        size_of::<JsCorrection>()
    );

    pub unsafe fn js_set_correction(
        fd: libc::c_int,
        data: &mut [JsCorrection],
    ) -> nix::Result<libc::c_int> {
        let res = libc::ioctl(fd, REQ_SET_CORRECTION, data);
        Errno::result(res)
    }

    pub unsafe fn js_get_correction(
        fd: libc::c_int,
        data: &mut [JsCorrection],
    ) -> nix::Result<libc::c_int> {
        let res = libc::ioctl(fd, REQ_GET_CORRECTION, data);
        Errno::result(res)
    }
}

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
            EventType::AXIS => {
                // assume value range is 0-255 (u8).
                let value = ev.value << 8;
                Some(Event::AxisChanged(ev.number, value))
            }
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
                use OpenError::*;

                match err {
                    Errno::ENOENT => DeviceFileNotFound(path.to_string()),
                    Errno::EPERM => PermissionDenied(path.to_string()),
                    Errno::EINVAL => InvalidPath(path.to_string()),
                    e => Unknown(e.into()),
                }
            },
        )?;

        Ok(Device(fd))
    }

    pub fn disable_correction(&self) -> Result<()> {
        let corr = unsafe {
            let mut axes = 0u8;
            ioctl::js_get_axes(self.0, &mut axes)?;
            let mut corr = vec![ioctl::JsCorrection::default(); axes as usize];
            ioctl::js_get_correction(self.0, corr.as_mut_slice())?;
            corr
        };

        let mut corr = corr
            .into_iter()
            .map(|mut c| {
                // disable calibration
                c.typ = CorrectionType::None;
                c
            })
            .collect::<Vec<_>>();

        unsafe {
            ioctl::js_set_correction(self.0, corr.as_mut_slice())?;
        };

        Ok(())
    }

    pub fn info(&self) -> Result<DeviceInfo> {
        let mut axes = 0u8;
        let mut buttons = 0u8;
        let mut name = [0u8; 128];

        unsafe {
            ioctl::js_get_axes(self.0, &mut axes)?;
            ioctl::js_get_buttons(self.0, &mut buttons)?;
            ioctl::js_get_name(self.0, &mut name)?;
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
            Err(Errno::ENODEV) => Some(Event::Disconnected),
            Err(e) => Some(Event::Error(format!("read error: {}", e))),
        }
    }
}
