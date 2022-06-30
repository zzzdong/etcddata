#!/usr/bin/env bash

OUT_DIR=etcddata-${GITHUB_REF_NAME}-${RUNNER_OS}-${RUNNER_ARCH}

RELEASE_DIR=build/release/

mkdir -pv ${RELEASE_DIR}/${OUT_DIR}

cargo build --release && cp -v target/release/etcddata ${RELEASE_DIR}/${OUT_DIR} && cd ${RELEASE_DIR} && tar czvf ${OUT_DIR}.tar.gz ${OUT_DIR} && rm -vr ${OUT_DIR}
