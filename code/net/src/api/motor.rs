use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DCMotorAction {
    Stop,
    SetThrottle(f32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StepperMotorAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MotorAction {
    DCMotor(DCMotorAction),
    StepperMotor(StepperMotorAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MotorCtl {
    InitMotor {
        /// which physical controller board to connect to.
        /// currently only accepts a value of zero since there is one board
        ctrl_board: usize,
        /// if using stepper motors, this only supports values of 0-1
        ///
        /// if using DC motors, this supports values of 0-3
        ///
        /// however, if using stepper motor 0, DC motors 0-1 become unavailable,
        /// and if using stepper motor 1, DC motors 2-3 become unavailable. the same is true for using DC motors.
        ///
        motor_id: motorkit_compat::Motor,
    },
    UpdateMotor {
        /// which physical controller board to connect to.
        /// currently only accepts a value of zero since there is one board
        ctrl_board: usize,
        /// if using stepper motors, this only supports values of 0-1
        ///
        /// if using DC motors, this supports values of 0-3
        ///
        /// however, if using stepper motor 0, DC motors 0-1 become unavailable,
        /// and if using stepper motor 1, DC motors 2-3 become unavailable. the same is true for using DC motors.
        ///
        motor_id: motorkit_compat::Motor,
        /// the action to perform
        ///
        /// the type of `MotorAction` must match the `MotorMode` of the referenced motor
        action: MotorAction,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MotorInfo {
    InitMotorError {
        motor: motorkit_compat::Motor,
        error: motorkit_compat::MotorError,
    },
    MotorInitialized {
        motor: motorkit_compat::Motor,
    },
    UpdateMotorError {
        motor: motorkit_compat::Motor,
        error: motorkit_compat::MotorError,
    },
}
