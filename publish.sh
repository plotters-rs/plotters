#!/bin/bash
set -e 

# Initialize the variables
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
MSRV=$(cat ${ROOT}/doc-template/msrv.txt)


echo "Publishing new version ${OLD_VERSION} -> ${NEW_VERSION}"

# Render new README
echo ${NEW_VERSION} > doc-template/latest_version
doc-template/update_readme.sh

# Then we need to verify the MSRV actually builds
rustup install ${MSRV}
cargo +${MSRV} build --features="svg_backend,chrono,image,deprecated_items,all_series,all_elements" --no-default-features

# Now we need to patch Cargo.toml
PATTERN=$(echo ^version = \"${OLD_VERSION}\"\$ | sed 's/\./\\./g')
DATE=$(date "+%Y-%m-%d")
for MANIFEST in */Cargo.toml
do
	sed -i "s/${PATTERN}/version = \"${NEW_VERSION}\"/g" Cargo.toml
	sed -i "s/path = \"\.\./[^\"]*\"/version = \"${NEW_VERSION}\""
done

exit 0

# In the end, we push tags and branches to remote

git add -u .
git commit -m "Bump version number from ${OLD_VERSION} to ${NEW_VERSION}"
git tag -a "v${NEW_VERSION}" -m "Plotters ${NEW_VERSION} release"

cargo publish
git push origin
git push origin "v${NEW_VERSION}"
