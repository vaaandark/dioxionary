#!/bin/bash
#set -xv

(
  cd "$(dirname "$0")"
  pj_dir=$(pwd)
  cargo build --release
  if [[ ! -d ~/.local/bin/ ]]; then
    echo make directory ~/.local/bin
    mkdir -p ~/.local/bin
  fi
  cp ./target/release/rmall ~/.local/bin/rmall
  which rmall > /dev/null
  if [[ $? != 0 ]]; then
    echo '~/.local/bin/ is not in your PATH' >&2
  fi
)

