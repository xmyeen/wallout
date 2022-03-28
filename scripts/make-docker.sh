#/bin/bash

DOCKER_BUILD_ROOT=".dockerbuild"
DOCKER_AF_NAME_DEF="wallout.tar.gz"
DOCKER_IMAGE_DEF="wallout:0.0.1"
DOCKER_FILE_DEF="wallout-dockerfile"

[ -e ${DOCKER_BUILD_ROOT} ] && rm -rf ${DOCKER_BUILD_ROOT}
mkdir -p ${DOCKER_BUILD_ROOT}/wallout

cd ${DOCKER_BUILD_ROOT}

cp -f ../target/release/{wallout-svr,libwallout.so} wallout &&
cp -rf ../config wallout &&
tar zcvf ${DOCKER_AF_NAME_DEF} wallout

cat > ${DOCKER_FILE_DEF} << EOF
FROM centos:7
ADD ${DOCKER_AF_NAME_DEF} /opt
RUN chmod +x /opt/wallout/wallout-svr
WORKDIR /opt/wallout
ENTRYPOINT ["/opt/wallout/wallout-svr", "./libwallout.so"]
EOF
docker rmi -f ${DOCKER_IMAGE_DEF}
docker build --force-rm  -t ${DOCKER_IMAGE_DEF} -f ${DOCKER_FILE_DEF} .

cd - &&
rm -rf ${DOCKER_BUILD_ROOT}