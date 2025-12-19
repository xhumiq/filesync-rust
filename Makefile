ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
SRC_DIR := ${ROOT_DIR}
APP_PROJ_NAME := ${proj}
ifeq ("${APP_PROJ_NAME}", "")
override APP_PROJ_NAME := webui
endif
PACKAGE_FOLDER := /ntc/packages/filesync/${APP_PROJ_NAME}
BUILD_PATH := ${ROOT_DIR}/${APP_PROJ_NAME}/target/release
S3_URL := https://us-lax-1.linodeobjects.com
S3_DEPLOY_PATH := s3://deploy/packages
S3_DEPLOY_PROFILE := /ntc/assets/.secrets/cloud-cli/lin_lax.s3cmd
AWS_DOCKER_REG := 362003809253.dkr.ecr.us-west-2.amazonaws.com
AWS_ECR_REGION := us-west-2
AWS_ECR_ACCOUNT := 362003809253

include ${DEVOPS_PATH}/scripts/vars/git_vars.mk
include ${DEVOPS_PATH}/scripts/utils.mk

pod-dav:
	@cd dufs \
	&& podman build --build-arg SIGNED_URL=$${signed_url} --build-arg FILE_NAME=dufs-v${BUILD_VER}.zst -t dufs:${BUILD_VER} -f Dockerfile-ecr-rel .

ecr-dav: pod-dav aws-login
	podman push dufs:${BUILD_VER} ${AWS_DOCKER_REG}/fserve/dufs:${BUILD_VER}
	podman push dufs:${BUILD_VER} ${AWS_DOCKER_REG}/fserve/dufs:latest

dep-dav:
	cd dufs \
	&& ${MAKE} release \
	&& mkdir -p ${PACKAGE_FOLDER} \
	&& rm -f ${PACKAGE_FOLDER}/dufs-v${BUILD_VER}.zst \
	&& rm -f ${PACKAGE_FOLDER}/dufs-latest.zst \
	&& zstd -o ${PACKAGE_FOLDER}/dufs-v${BUILD_VER}.zst ${ROOT_DIR}/dufs/target/release/dufs  \
	&& cp ${PACKAGE_FOLDER}/dufs-v${BUILD_VER}.zst ${PACKAGE_FOLDER}/dufs-latest.zst \
	&& ls -al ${PACKAGE_FOLDER} \
	&& s5cmd --endpoint-url ${S3_URL} \
		--profile deploy \
		--credentials-file ${HOME}/.config/cloud-cli/.linode \
		cp ${PACKAGE_FOLDER}/dufs-latest.zst ${S3_DEPLOY_PATH}/dufs/dufs-v${BUILD_VER}.zst

dep-ui:
	$(MAKE) deploy-ui proj=webui apihost=${apihost}

deploy-ui:
	cd webui && rm -rf dist/release dist/webui \
	&& ${MAKE} release apihost=${apihost}
	mkdir -p ${PACKAGE_FOLDER}
	rm -f ${PACKAGE_FOLDER}/webui-${apihost}-v${BUILD_VER}.tar.zst
	rm -f ${PACKAGE_FOLDER}/webui-${apihost}-latest.tar.zst
	cd webui/dist \
	&& rm -rf webui && mv release webui \
	&& tar -I zstd -cvf ${PACKAGE_FOLDER}/webui-${apihost}-v${BUILD_VER}.tar.zst webui \
	&& rm -rf release && mv webui release
	cp ${PACKAGE_FOLDER}/webui-${apihost}-v${BUILD_VER}.tar.zst ${PACKAGE_FOLDER}/webui-${apihost}-latest.tar.zst
	ls -al ${PACKAGE_FOLDER}
	s5cmd --endpoint-url ${S3_URL} \
		--profile deploy \
		--credentials-file ${HOME}/.config/cloud-cli/.linode \
		cp ${PACKAGE_FOLDER}/webui-${apihost}-latest.tar.zst ${S3_DEPLOY_PATH}/${APP_PROJ_NAME}/${APP_PROJ_NAME}-${apihost}-v${BUILD_VER}.zst
	s5cmd --endpoint-url ${S3_URL} \
		--profile deploy \
		--credentials-file ${HOME}/.config/cloud-cli/.linode \
		cp ${PACKAGE_FOLDER}/webui-${apihost}-latest.tar.zst ${S3_DEPLOY_PATH}/${APP_PROJ_NAME}/${APP_PROJ_NAME}-${apihost}-latest.zst
	s5cmd --endpoint-url ${S3_URL} \
		--profile deploy \
		--credentials-file ${HOME}/.config/cloud-cli/.linode \
		cp ${PACKAGE_FOLDER}/webui-${apihost}-latest.tar.zst ${S3_DEPLOY_PATH}/${APP_PROJ_NAME}/${APP_PROJ_NAME}-v${BUILD_VER}.zst
	s5cmd --endpoint-url ${S3_URL} \
		--profile deploy \
		--credentials-file ${HOME}/.config/cloud-cli/.linode \
		cp ${PACKAGE_FOLDER}/webui-${apihost}-latest.tar.zst ${S3_DEPLOY_PATH}/${APP_PROJ_NAME}/${APP_PROJ_NAME}-latest.zst

dep-fs:
	$(MAKE) deploy-fs proj=webfs

deploy-fs:
	cd webfs \
	&& ${MAKE} release \
	&& mkdir -p ${PACKAGE_FOLDER} \
	&& rm -f ${PACKAGE_FOLDER}/webfs-v${BUILD_VER}.zst \
	&& rm -f ${PACKAGE_FOLDER}/webfs-latest.zst \
	&& zstd -o ${PACKAGE_FOLDER}/webfs-v${BUILD_VER}.zst ${BUILD_PATH}/${APP_PROJ_NAME}  \
	&& cp ${PACKAGE_FOLDER}/webfs-v${BUILD_VER}.zst ${PACKAGE_FOLDER}/webfs-latest.zst \
	&& ls -al ${PACKAGE_FOLDER} \
	&& s5cmd --endpoint-url ${S3_URL} \
		--profile deploy \
		--credentials-file ${HOME}/.config/cloud-cli/.linode \
		cp ${PACKAGE_FOLDER}/webfs-latest.zst ${S3_DEPLOY_PATH}/${APP_PROJ_NAME}/${APP_PROJ_NAME}-v${BUILD_VER}.zst

aws-login:
	@aws ecr get-login-password --region ${AWS_ECR_REGION} | \
		podman login --username AWS --password-stdin ${AWS_DOCKER_REG}
