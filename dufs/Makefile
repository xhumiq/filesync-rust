ROOT_DIR := $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
SRC_DIR := ${ROOT_DIR}

include ${DEVOPS_PATH}/scripts/vars/git_vars.mk
include ${DEVOPS_PATH}/scripts/utils.mk

new_msg := Update On Build
tag_msg := ${new_msg}
deploy_env := Local Build Testing Only

BUILD_PATH := /ntc/builds/caddy/caddy
CADDY_VERSION := 2.10.2

CGO_ENABLED := 0
GOPRIVATE := bitbucket.org/xhumiq
BUILD_FLAGS := -X main.appId=${PROJ_APP_GUID} -X main.version=${BUILD_VER} -X main.sourceTag=${SOURCE_TAG} -X 'main.commitMsg=${COMMIT_MSG}' -X main.gitHash=${SOURCE_COMMIT} -X 'main.branch=${BRANCH}' -X main.buildStamp=${BUILD_TIME} -X 'main.awsKey=${AWS_SECRET_ACCESS_KEY}' -X 'main.sqlPwd=${PROJ_SQL_PWD}' -X 'main.smtpPwd=${PROJ_SMTP_PWD}' -X 'main.jwtSecret=${JWT_SECRET}'

docker_reg := 362003809253.dkr.ecr.us-west-2.amazonaws.com
AWS_ECR_REGION := us-west-2
AWS_ECR_ACCOUNT := 362003809253
defPort := 5010
dk_app_prefix := "cmd"
export

build:
	cargo build

release:
	cargo build --release

pod-push: pod-build ecr-push

pod-build:
	@echo "Build Image DUFS Version: v${BUILD_VER} ${BUILD_PATH}" \
	; podman build . --cache-ttl=0 --pull -t dufs:v${BUILD_VER} \
	  --file Dockerfile-ecr-rel \
		&& podman run -it --rm -v ${BUILD_PATH}:/app dufs:v${BUILD_VER} cp /bin/dufs /app/dufs-v${BUILD_VER} \
		&& podman tag dufs:v${BUILD_VER} dufs:latest \
		&& podman tag dufs:v${BUILD_VER} dufs \
		&& echo Set dufs:v${BUILD_VER} to latest

aws-login:
	@aws ecr get-login-password --region ${AWS_ECR_REGION} | \
		podman login --username AWS --password-stdin ${docker_reg}

ecr-push: aws-login
	@echo "Pushing Image DUFS dufs:v${BUILD_VER}"
	podman push dufs:v${BUILD_VER} ${AWS_DOCKER_REG}/fserve/dufs:v${BUILD_VER}
	podman push dufs:v${BUILD_VER} ${AWS_DOCKER_REG}/fserve/dufs:latest
