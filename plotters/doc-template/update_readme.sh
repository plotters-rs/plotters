#!/bin/bash
REPO_BASE=`readlink -f $(dirname $(readlink -f $0))/../`
${REPO_BASE}/doc-template/render_readme.sh ${REPO_BASE}/doc-template/readme.template.md ${REPO_BASE}/doc-template/readme > ${REPO_BASE}/README.md

awk '
NR == FNR {
	doc = doc"\n"$0;
}
NR != FNR{
	if($0 == "/*!") {
		in_doc = 1;
	}
	if(!in_doc) {
		print $0
	}
	if($0 == "*/") {
		print "/*!"
		print doc
		print "*/"
		in_doc = 0;
	}
}' <(${REPO_BASE}/doc-template/render_readme.sh ${REPO_BASE}/doc-template/readme.template.md ${REPO_BASE}/doc-template/rustdoc) ${REPO_BASE}/src/lib.rs > ${REPO_BASE}/src/lib.rs.tmp

mv ${REPO_BASE}/src/lib.rs.tmp ${REPO_BASE}/src/lib.rs
cargo fmt
