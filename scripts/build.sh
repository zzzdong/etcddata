#!/usr/bin/env bash

RELEASE_DIR=build/release/etcddata-${GITHUB_REF_NAME}-${RUNNER_OS}-${RUNNER_ARCH}

mkdir -pv ${RELEASE_DIR}

cargo build --release && cp -v target/release/etcddata ${RELEASE_DIR} && tar czvf ${RELEASE_DIR}.tar.gz ${RELEASE_DIR}
