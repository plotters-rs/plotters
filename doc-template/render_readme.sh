#!/bin/sh
DIR=$2
awk '{
	if($0 ~ /\$\$.*\$\$/) {
		filename = "'${DIR}/'"substr($0,3,length($0)-4);
		while((getline content < filename) > 0) {
			print content;
		}
	} else {
		print $0;
	}
}
' $1 | sed 's/\$LATEST_VERSION/'$(cat ${DIR}/../latest_version)'/g'
