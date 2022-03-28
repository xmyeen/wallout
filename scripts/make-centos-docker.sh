#/bin/bash

WORKDIR="${PWD}"
DOCKER_BUILD_ROOT=".dockerbuild"
DOCKER_AF_NAME_DEF="wallout.tar.gz"
DOCKER_IMAGE_DEF="wallout-centos7:0.0.2"
DOCKER_FILE_DEF="wallout-dockerfile"

[ -e ${DOCKER_BUILD_ROOT} ] && rm -rf ${DOCKER_BUILD_ROOT}
mkdir -p ${DOCKER_BUILD_ROOT}/wallout

cd ${DOCKER_BUILD_ROOT}

cp -vf ${WORKDIR}/.dist/*.tar.gz .

cat > ${DOCKER_FILE_DEF} << EOF
FROM centos:7
ADD ${DOCKER_AF_NAME_DEF} /opt
RUN yum install -y openssl-devel && yum clean all && chmod +x /opt/wallout/wallout-svr
WORKDIR /opt/wallout
ENTRYPOINT ["/opt/wallout/wallout-svr"]
EOF
docker rmi -f ${DOCKER_IMAGE_DEF}
docker build --force-rm  -t ${DOCKER_IMAGE_DEF} -f ${DOCKER_FILE_DEF} .

cd - &&
rm -rf ${DOCKER_BUILD_ROOT}