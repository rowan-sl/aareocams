#!/bin/bash

#* DO NOT run this by hand, run this through the makefile

source ./config/deploy-config.sh

rsync ./build/$AAREOCAMS_DEPLOY_BUILD_MODE/aareocams-bot $AAREOCAMS_DEPLOY_TARGET_UNAME@$AAREOCAMS_DEPLOY_TARGET_IP:$AAREOCAMS_DEPLOY_TARGET_PATH
