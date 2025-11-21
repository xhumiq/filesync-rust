ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
SRC_DIR := ${ROOT_DIR}
APP_PROJ_NAME := ${proj}
ifeq ("${APP_PROJ_NAME}", "")
override APP_PROJ_NAME := webui
endif
PACKAGE_FOLDER := "/ntc/packages/filesync/${APP_PROJ_NAME}"
BUILD_PATH := ${ROOT_DIR}/${APP_PROJ_NAME}/target/release
S3_URL := https://us-lax-1.linodeobjects.com
S3_DEPLOY_PATH := s3://deploy/packages

include ${DEVOPS_PATH}/scripts/vars/git_vars.mk
include ${DEVOPS_PATH}/scripts/utils.mk

dep-ui:
	$(MAKE) deploy-ui proj=webui

deploy-ui:
	cd webui \
	&& ${MAKE} release \
	&& mkdir -p ${PACKAGE_FOLDER} \
	&& rm -f ${PACKAGE_FOLDER}/webui-v${BUILD_VER}.tar.zst \
	&& rm -f ${PACKAGE_FOLDER}/webui-latest.tar.zst \
	&& tar -I zstd -cvf ${PACKAGE_FOLDER}/webui-v${BUILD_VER}.tar.zst dist \
	&& cp ${PACKAGE_FOLDER}/webui-v${BUILD_VER}.tar.zst ${PACKAGE_FOLDER}/webui-latest.tar.zst \
	&& ls -al ${PACKAGE_FOLDER} \
	&& s5cmd --endpoint-url ${S3_URL} \
		--profile deploy \
		--credentials-file ${HOME}/.config/cloud-cli/.linode \
		cp ${PACKAGE_FOLDER}/webui-latest.tar.zst ${S3_DEPLOY_PATH}/${APP_PROJ_NAME}/${APP_PROJ_NAME}-v${BUILD_VER}.zst

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
