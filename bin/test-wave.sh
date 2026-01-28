#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

source "${BIN}/lib-verbose.sh"

if [[ "$#" -lt 1 ]]
then
    set -- 34
fi

mkdir -p "${PROJECT}/data/tmp/wave"
(
    cd "${PROJECT}"
    rm -f data/tmp/wave/*.{pgm,png}
    ls -al data/tmp/wave
    export RUST_LOG=debug
    export RUST_BACKTRACE=1
    cargo run wave "$@" --export-dir data/tmp/wave

    cd data/tmp/wave
    LAST_IMAGE="$(ls -1d *.pgm | sort -t - -n -k 2 | tail -1)"
    PNG="${LAST_IMAGE%.pgm}.png"
    info "Convert: [${LAST_IMAGE}] -> [${PNG}]"
    DATA_DIR="$(pwd)"
    docker run --rm -v "${DATA_DIR}:${DATA_DIR}" -w "${DATA_DIR}" jeroenvm/wrapper-netpbm pnmtopng "${LAST_IMAGE}" > "${PNG}"
) 2>&1 | tee "${PROJECT}/data/tmp/wave/quantized-interactions.log"
