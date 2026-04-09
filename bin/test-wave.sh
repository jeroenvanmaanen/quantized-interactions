#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

source "${BIN}/lib-verbose.sh"

CARGO_FLAGS=()

if "${TRACE}"
then
    LOG_LEVEL='trace'
elif "${SILENT}"
then
    LOG_LEVEL='info'
    CARGO_FLAGS+='--release'
else
    LOG_LEVEL='debug'
fi

if [[ "$#" -lt 1 ]]
then
    set -- 60
fi

mkdir -p "${PROJECT}/data/tmp/wave"
(
    cd "${PROJECT}"
    rm -f data/tmp/wave/*.{pgm,png}
    ls -al data/tmp/wave
    export RUST_LOG="${LOG_LEVEL}"
    export RUST_BACKTRACE=1
    cargo run "${CARGO_FLAGS[@]}" wave "$@" --export-dir data/tmp/wave
) 2>&1 | tee "${PROJECT}/data/tmp/wave/quantized-interactions.log"
