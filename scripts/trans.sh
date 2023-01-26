#!/bin/bash

# Usage:
#   ./trans.sh {WORD}
# Modify the following array to set the priority
dicts=(langdao-ec-gb oxford-gb cdict-gb kdic-computer-gb)

word="$1"

for i in "${dicts[@]}"; do
  if rmall lookup -l "$HOME/.config/rmall/$i" -e "$word"; then
    exit 0
  fi
done

rmall lookup "$word"
