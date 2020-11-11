use std::fmt;

pub enum DeviceError {
    DeviceNotFound
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeviceError::DeviceNotFound =>
                write!(f, "device not found"),
        }
    }
}