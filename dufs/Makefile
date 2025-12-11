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

pod-build:
	@echo "Build Image Caddy Version: ${CADDY_VERSION} ${BUILD_PATH}" \
	; cd ../../ \
	; pwd \
	; podman build . --cache-ttl=0 --pull -t caddy:${CADDY_VERSION} \
	  --file ./plugins/caddy-keycloak/Dockerfile \
		&& podman tag caddy:${CADDY_VERSION} caddy:latest \
		&& podman tag caddy:${CADDY_VERSION} caddy \
		&& echo Set caddy:${CADDY_VERSION} to latest

aws_login:
	@aws ecr get-login-password --region ${AWS_ECR_REGION} | \
		podman login --username AWS --password-stdin ${docker_reg}

ecr_push: aws_login
	@echo "BUILD_VER=${CADDY_VERSION}"
	podman push caddy:${CADDY_VERSION} ${docker_reg}/auth/caddy:${CADDY_VERSION}
	podman push caddy:${CADDY_VERSION} ${docker_reg}/auth/caddy:latest
