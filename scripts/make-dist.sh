#!/bin/sh
DISTDIR='.dist'
BUILDDIR='.build'

rm -rf ${DISTDIR} && mkdir -p ${BUILDDIR}/wallout/{config,data} ${DISTDIR}
cp -vf ./target/release/wallout-svr ${BUILDDIR}/wallout &&
cp -vf ./config/*.toml ${BUILDDIR}/wallout/config/ &&
cp -vf ./config/*.yml ${BUILDDIR}/wallout/config/ &&
cp -vf ./config/*.yaml ${BUILDDIR}/wallout/config/ &&
cp -vf ./data/*.crt ${BUILDDIR}/wallout/data/ &&
cp -vf ./data/*.key ${BUILDDIR}/wallout/data/ &&
tar zcvf ${DISTDIR}/wallout.tar.gz -C ${BUILDDIR} wallout
rm -rf ${BUILDDIR}