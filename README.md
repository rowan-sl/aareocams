# AAREOCAMS (Aerial Autonomous and REmotely Operated CAble Monorail System)

## About this repo

This contains the code for aareocams, and may in the future contain information about the physical robot.

Code for the robot is run on a raspberry pi, and the controll are run on a desktop, connecting over wifi.
this currently will only support the sn30pro controller, as that is what i have, but may support more in the future

## About the robot

note to future me: fill this in with real information

## Project Structure

This is set up as a cargo workspace, with different sub-programs and libraries as organization, each with their own directory which is listed here

- `bot`: final executable which is placed on the robot
- `dash`: final dashboard executable for controlling the robot
- `net`: project spacific networking code
- `scomm`: more general networking code, you can use this in your own project if you want
- `core`: logic and code used by **all** crates in this project, depends on no other crate in this project

## Configuration

see [config](config/README.md)

## Compiling

currently there is realy no point, as you need the robot to use it ¯\\\_(ツ)_/¯ but in order to build the code you need a few things

- cargo, with the `armv7-unknown-linux-gnueabihf` toolchain installed (see [toolchain](##Toolchain))
- `gcc-arm-linux-gnueabihf` (for linking)
- gstreamer (for installation instructions, see the [gstreamer-rs build instructions](https://github.com/sdroege/gstreamer-rs#installation))
- opencv (see the [rust opencv library github](https://github.com/twistedfall/opencv-rust) for installation details)
- probably something else that is missing (make a issue on github if there is)

to build, use `make all` to build all targets, or one of the many available commands listed here:

- `make debug` build the debug target
- `make release` build the release target
- `make clean` clean up all build artifacts
- `make deploy_r` or `make deploy_d`: see the [deploying](##Deploying) section

the produced executables will be moved to subdirectories in the `build/` based on what target they were built for.

please note that the `aareocams-bot` executables will have been built for the raspberry pi (`armv7-unknown-linux-gnueabihf`) target.

## Deploying

Deployment requires a few extra steps

- make shure you have rsync installed, for transfering the files
- a raspberry pi (tested on the raspberry pi 3 B+) to deploy to
- configure ssh so you can connect to the pi you are deploying to

To deploy, make shure the pi is running and run `make deploy_r` or `make deploy_d` based on if you want to deploy release or debug code

## Toolchain

Configuring the raspberry pi cross compilation toolchain is annoying at best, so it is recommended to use [this build script collection](https://github.com/abhiTronix/raspberry-pi-cross-compilers).

To start, run `make build-toolchain`. Currently all it does is download the repo into the [toolchain](toolchain) directory, but this will be improved in the future to do more of this process.

after this, run the toolchain build script ([instructions](toolchain/raspberry-pi-cross-compilers/build-scripts/README.md)) and copy the resulting executables into the `toolchain/bin/` directory.

## TODO's

- [ ] allow querying and configuration of camera formats from the dashboard, including resolution and FPS (and combinations of bolth)
- [ ] allow querying which cameras are available from the dashboard
- [x] fix dashboard so that it uses the new `Uuid` stream identification system
- [ ] add configurable options for bits per packet when initializing a camera stream (lvenc encoder option)
- [x] **IMPORTANT** make `deploy.sh` move the bot config files as well as the executable!
- [ ] make a tool for setting up configuration
- [ ] set up encrypted TLS connections
- [ ] add build script support for building and deploying subsystems
- [ ] figure out if can bus is a viable option for communicating with subsystems instead of i2c
- [ ] get a different motor hat then the current one :angre:
- [ ] UPDATE ICED VERSION!!!!!
