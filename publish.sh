#!/bin/bash
set -e 

if [ $(git diff | wc -l) != 0 ]
then
	echo "Commit your change before publish!"
	exit 1
fi

if [ -z "$1" ]
then
	PART_ID=0
else
	PART_ID=$1
fi
ROOT=$(dirname $(readlink -f $0))
OLD_VERSION=$(cat ${ROOT}/doc-template/latest_version)
NEW_VERSION=$(awk -F. 'BEGIN{idx=3-'${PART_ID}'}{$idx+=1; for(i=idx+1; i <= 3; i++) $i=0; print $1"."$2"."$3}' ${ROOT}/doc-template/latest_version)
echo "Publishing new version ${OLD_VERSION} -> ${NEW_VERSION}"
echo ${NEW_VERSION} > doc-template/latest_version
doc-template/update_readme.sh
echo ${OLD_VERSION} > doc-template/latest_version
cargo fmt

PATTERN=$(echo ^version = \"${OLD_VERSION}\"\$ | sed 's/\./\\./g')
DATE=$(date "+%Y-%m-%d")
sed -i "s/${PATTERN}/version = \"${NEW_VERSION}\"/g" Cargo.toml
PATTERN=$(echo ${NEW_VERSION} | sed 's/\./\\./g')
sed -i "s/^## Plotters .* (?) *\$/## Plotters ${NEW_VERSION} ($DATE)/g" CHANGELOG.md

echo ${NEW_VERSION} > doc-template/latest_version

git add -u .
git commit -m "Bump version number from ${OLD_VERSION} to ${NEW_VERSION}"
git tag -a "v${NEW_VERSION}" -m "Plotters ${NEW_VERSION} release"

# Verify MSRV
MSRV=$(cat ${ROOT}/doc-template/msrv.txt)
rustup install ${MSRV}
cargo +${MSRV} build

cargo publish
git push origin
git push origin "v${NEW_VERSION}"
