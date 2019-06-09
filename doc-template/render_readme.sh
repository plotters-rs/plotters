#!/bin/sh
DOC_ROOT=$(readlink -f $(dirname $0))
DIR=$2
TOC_FILENAME=$(mktemp)

${DOC_ROOT}/gen-toc.sh $1 > ${TOC_FILENAME}

awk '{
	if($0 ~ /\$\$.*\$\$/) {
		filename = substr($0,3,length($0)-4);
		if(filename == "[TOC]") {
			filename ="'${TOC_FILENAME}'"
		} else {
			filename = "'${DIR}/'"filename
		}
		while((getline content < filename) > 0) {
			print content;
		}
	} else {
		print $0;
	}
}
' $1 | sed 's/\$LATEST_VERSION/'$(cat ${DOC_ROOT}/latest_version)'/g'

rm -f ${TOC_FILENAME}
