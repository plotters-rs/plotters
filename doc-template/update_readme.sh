#!/bin/bash
REPO_BASE=`readlink -f $(dirname $(readlink -f $0))/../`
${REPO_BASE}/doc-template/render_readme.sh ${REPO_BASE}/doc-template/readme.template.md ${REPO_BASE}/doc-template/readme > ${REPO_BASE}/README.md
