#!/bin/bash
export PATH=/usr/bin

mkdir -p dockertarget

docker run --rm --mount type=bind,src=$PWD,target=/usr/src/textperms -it rust:slim-bullseye "/bin/bash" "-c" "cd /usr/src/textperms; rustup install nightly; rustup override set nightly; cargo build -Z unstable-options --release --out-dir=dockertarget"
