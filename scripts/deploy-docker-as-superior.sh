#/bin/bash

IMAGE_NAME="wallout:0.0.2"

HOST_WORKDIR="${PWD}"
HOST_NAME="localhost"
HOST_PORT="443"
HOST_CFG="app.toml"

WORDIR="/opt/wallout"

CR_USER="admin"
CR_PASS="gh89"

echo "Set work directory as '${HOST_WORKDIR}'"

cat > ${HOST_WORKDIR}/${HOST_CFG} << EOF
proxy_pass_locations = []

[client]
certfile = "data/client.crt"
keyfile = "data/client.key"
trusted_cerfiles = []

[[servers]]
id = "https"
host = "0.0.0.0"
port = ${HOST_PORT}
realm = "wallout"
on_https = true
certfile = "data/server.crt"
keyfile = "data/server.key"

[[credentials]]
user = "${CR_USER}"
passwd = "${CR_PASS}"

[secure]
whitelists = []

[tunnal]
superiors = []
EOF

mkdir -p ${HOST_WORKDIR}
docker run \
    -d --name wallout  \
    -v ${HOST_WORKDIR}/${HOST_CFG}:${WORDIR}/config/app.toml \
    -v ${HOST_WORKDIR}/ssl/client.crt:${WORDIR}/data/client.crt:ro \
    -v ${HOST_WORKDIR}/ssl/client.key:${WORDIR}/data/client.key:ro \
    -v ${HOST_WORKDIR}/ssl/server.crt:${WORDIR}/data/server.crt:ro \
    -v ${HOST_WORKDIR}/ssl/server.key:${WORDIR}/data/server.key:ro \
    -w ${WORDIR} \
    -p ${HOST_PORT}:${HOST_PORT} \
    ${IMAGE_NAME}