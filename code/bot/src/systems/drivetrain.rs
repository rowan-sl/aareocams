// use std::{thread::{self, JoinHandle}, time::Duration};

// use adafruit_motorkit::{init_pwm, Motor, stepper::{StepperMotor, StepDirection, StepStyle}};
// use flume::Sender;
// use tokio::sync::oneshot;


// pub const DRIVE_M0: Motor = Motor::Stepper1;

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum MotorAction {
//     Fwd,
//     Back,
//     Stop,
// }

// pub struct SuperHackyStepperDriver {
//     busyloop_thread: JoinHandle<()>,
//     kill_channel: oneshot::Sender<()>,
//     action_channel: Sender<MotorAction>,
// }

// impl SuperHackyStepperDriver {
//     pub fn new() -> Self {
//         let (kill_channel, mut kill_sig) = oneshot::channel();
//         let (action_channel, action_recv) = flume::unbounded();
//         let busyloop_thread = thread::spawn(move || {
//             let mut driver = init_pwm(None).unwrap();
//             let mut drive_m = StepperMotor::try_new(&mut driver, DRIVE_M0, None).unwrap();
//             let mut drive_state = MotorAction::Stop;
//             while let Err(oneshot::error::TryRecvError::Empty) = kill_sig.try_recv() {
//                 if let Ok(action) = action_recv.try_recv() {
//                     if action == MotorAction::Stop {
//                         drive_m.stop(&mut driver).unwrap();
//                     }
//                     drive_state = action;
//                 }
//                 match drive_state {
//                     MotorAction::Stop => {
//                         std::thread::sleep(Duration::from_millis(50));
//                     }
//                     MotorAction::Fwd => {
//                         drive_m.step_once(&mut driver, StepDirection::Forward, StepStyle::Single).unwrap();
//                     }
//                     MotorAction::Back => {
//                         drive_m.step_once(&mut driver, StepDirection::Backward, StepStyle::Single).unwrap();
//                     }
//                 }
//             }
//         });
//         Self {
//             busyloop_thread,
//             kill_channel,
//             action_channel,
//         }
//     }

//     pub fn set_dir(&mut self, dir: MotorAction) {
//         self.action_channel.send(dir).unwrap();
//     }

//     pub fn close(self) {
//         self.kill_channel.send(()).unwrap();
//         self.busyloop_thread.join().unwrap();
//     }
// }
