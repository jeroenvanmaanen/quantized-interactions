#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

source "${BIN}/lib-verbose.sh"

COMMAND="$(basename "$0" '.sh')"
OUTPUT="${PROJECT}/data/tmp/${COMMAND}"

mkdir -p "${OUTPUT}"
(
    cd "${PROJECT}"
    rm -f "${OUTPUT}"/*.{pgm,png}
    ls -al "${OUTPUT}"
    export RUST_LOG=debug
    export RUST_BACKTRACE=1
    cargo run "${COMMAND}" "$@" --export-dir "${OUTPUT}"
) 2>&1 | tee "${OUTPUT}/quantized-interactions.log"
