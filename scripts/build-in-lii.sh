#!/bin/sh

HOST_WORKDIR="${PWD}"
WORKDIR="/opt/walout"
DISTDIR=".dist"

BUILD_SCRIPT="b.sh"

cat > ${HOST_WORKDIR}/${BUILD_SCRIPT} << EOF
yum intall openssel-devel
echo "" > \$HOME/.cargo/config
source > \$HOME/.cargo/env
cargo b --release
sh scripts/make-dist.sh
EOF

docker run \
    --rm \
    -it \
    -v ${PWD}:${WORKDIR} \
    -w ${WORKDIR} \
    lii:latest \
    sh ${WORKDIR}/${BUILD_SCRIPT}

rm -f ${HOST_WORKDIR}/${BUILD_SCRIPT}



