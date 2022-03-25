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

## Compiling

currently there is realy no point, as you need the robot to use it ¯\\\_(ツ)_/¯ but in order to build the code you need a few things

- cargo, with the armv7-unknown-linux-gnueabihf toolchain installed
- gcc-arm-linux-gnueabihf (for linking)
- probably something else that is missing (make a issue on github if there is)

to build, use `make all` to build all targets, or one of the many available commands listed here:

- `make debug` build the debug target
- `make release` build the release target
- `make clean` clean up all build artifacts

the produced executables will be moved to subdirectories in the `build/` based on what target they were built for.
