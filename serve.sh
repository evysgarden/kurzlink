#!/usr/bin/env sh

KL_PWD="$(pwd)"
KL_BASEPATH="${KL_PWD}/servetest"
KL_VANITYPATH="${KL_BASEPATH}/vanitymap.json"
KL_REDIRECTFILE="${KL_BASEPATH}/_redirects"
KL_CONFIGFILE="${KL_PWD}/kurzlink.yml"

rm -rf "${KL_BASEPATH}"
cargo run -- -g -o "${KL_BASEPATH}" -m "${KL_VANITYPATH}" -c "${KL_CONFIGFILE}" -r "${KL_REDIRECTFILE}" -n || exit
cd "${KL_BASEPATH}"
python3 -m http.server
cd "${KL_PWD}"
rm -rf "${KL_BASEPATH}"
