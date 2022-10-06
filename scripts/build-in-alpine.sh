#!/bin/sh

HOST_WORKDIR="${PWD}"
WORKDIR="/opt/walout"
BUILD_SCRIPT="b.sh"

cat > ${HOST_WORKDIR}/${BUILD_SCRIPT} << EOF
apk add build-base &&
apk add openssl-dev &&
apk add curl &&
curl --proto '=https' -o /tmp/rust-init.sh --tlsv1.2 -sSf https://sh.rustup.rs &&
sh /tmp/rust-init.sh -y && 
source \$HOME/.cargo/env &&
cargo b --release
sh scripts/make-dist.sh
EOF

docker run \
    --rm \
    -it \
    -v ${PWD}:${WORKDIR} \
    -w ${WORKDIR} \
    alpine:latest \
    sh ${WORKDIR}/${BUILD_SCRIPT}

rm -f ${HOST_WORKDIR}/${BUILD_SCRIPT}