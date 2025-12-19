ROOT_DIR := $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
SRC_DIR := ${ROOT_DIR}

include ${DEVOPS_PATH}/scripts/vars/git_vars.mk
include ${DEVOPS_PATH}/scripts/utils.mk

new_msg := Update On Build
tag_msg := ${new_msg}
deploy_env := Local Build Testing Only

BUILD_PATH := /ntc/builds/filesvr/webfs
docker_reg := 362003809253.dkr.ecr.us-west-2.amazonaws.com
AWS_ECR_REGION := us-west-2
AWS_ECR_ACCOUNT := 362003809253
export

ENV_FILE:=.env.local

.PHONY: build run

build:
	ENV_PROFILE=$(ENV_FILE) cargo build --bin webfs

utils:
	ENV_PROFILE=$(ENV_FILE) cargo build --bin utils
	RUST_BACKTRACE=1 ENV_PROFILE=$(ENV_FILE) ./target/debug/utils

release:
	cargo build --bin webfs --release

webfs: build
	RUST_BACKTRACE=1 ENV_PROFILE=$(ENV_FILE) ./target/debug/webfs --config=/ntc/filesync/rust/webfs/config-test.yaml

rel-webfs: release
	ENV_PROFILE=$(ENV_FILE) ./target/release/webfs --config=/ntc/filesync/rust/webfs/config.yaml

pod-build:
	@echo "Build Image Webfs Version: v${BUILD_VER} ${BUILD_PATH}"
	podman build . --cache-ttl=0 --pull -t webfs:v${BUILD_VER} --file Dockerfile
	podman tag webfs:v${BUILD_VER} webfs:latest \
		&& podman tag webfs:v${BUILD_VER} webfs \
		&& echo Set webfs:v${BUILD_VER} to latest

# localhost/webfs:v0.0.0-0
# podman run -it --rm -v ${BUILD_PATH}:/app webfs:v${BUILD_VER} cp /bin/webfs /app/webfs-v${BUILD_VER}

aws-login:
	@aws ecr get-login-password --region ${AWS_ECR_REGION} | \
		podman login --username AWS --password-stdin ${docker_reg}

ecr-push: aws-login
	@echo "Pushing Image DUFS webfs:v${BUILD_VER}"
	podman push webfs:v${BUILD_VER} ${AWS_DOCKER_REG}/fserve/webfs:v${BUILD_VER}
	podman push webfs:v${BUILD_VER} ${AWS_DOCKER_REG}/fserve/webfs:latest
