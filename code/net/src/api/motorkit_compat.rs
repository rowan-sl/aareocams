//! copy of structs from adafruit_motorkit, but that implement Serialize

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
/// A list of all errors that can be thrown by the library.
pub enum MotorError {
    /// An error occurred initializing the I2C bus.
    I2cError,
    /// An error occurred configuring the PCA9685.
    PwmError,
    /// An error occurred setting a channel.
    ChannelError,
    /// The value for throttle is not in the bounds of [-1.0, 1.0].
    ThrottleError,
    /// An invalid motor was provided to a constructor, i.e. a stepper motor
    /// passed into the DcMotor constructor.
    InvalidMotorError,
}

impl fmt::Display for MotorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<adafruit_motorkit::MotorError> for MotorError {
    fn from(other: adafruit_motorkit::MotorError) -> Self {
        match other {
            adafruit_motorkit::MotorError::I2cError => Self::I2cError,
            adafruit_motorkit::MotorError::PwmError => Self::PwmError,
            adafruit_motorkit::MotorError::ChannelError => Self::ChannelError,
            adafruit_motorkit::MotorError::ThrottleError => Self::ThrottleError,
            adafruit_motorkit::MotorError::InvalidMotorError => Self::InvalidMotorError,
        }
    }
}

impl std::error::Error for MotorError {}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
/// An enumeration of all potential motors that can be controlled via the
/// Motor HAT.
pub enum Motor {
    Motor1,
    Motor2,
    Motor3,
    Motor4,
    Stepper1,
    Stepper2,
}

impl From<adafruit_motorkit::Motor> for Motor {
    fn from(other: adafruit_motorkit::Motor) -> Self {
        match other {
            adafruit_motorkit::Motor::Motor1 => Self::Motor1,
            adafruit_motorkit::Motor::Motor2 => Self::Motor2,
            adafruit_motorkit::Motor::Motor3 => Self::Motor3,
            adafruit_motorkit::Motor::Motor4 => Self::Motor4,
            adafruit_motorkit::Motor::Stepper1 => Self::Stepper1,
            adafruit_motorkit::Motor::Stepper2 => Self::Stepper2,
        }
    }
}
