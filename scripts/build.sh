#!/usr/bin/env bash

OUT_DIR=etcddata-${GITHUB_REF_NAME}-x86_64-unknown-linux-musl

RELEASE_DIR=build/release/

mkdir -pv ${RELEASE_DIR}/${OUT_DIR}

cargo build --release --target=x86_64-unknown-linux-musl && cp -v target/release/etcddata ${RELEASE_DIR}/${OUT_DIR} && cd ${RELEASE_DIR} && tar czvf ${OUT_DIR}.tar.gz ${OUT_DIR} && rm -vr ${OUT_DIR}
