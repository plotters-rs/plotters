#!/bin/bash
 sed -n 's/^##\(#* .*\)$/\1/gp' $1 \
	 | sed -e 's/^ /1\t/g' -e 's/^# /2\t/g' \
	 | awk -F'\t' '
BEGIN {
	level[1] = "* ";
	level[2] = "  + ";
	print "## Table of Contents"
}
{
	link=tolower($2)
	gsub(/[^a-z0-9]/, "-", link);
	print "  "level[$1]"["$2"](#"link")"
}'
