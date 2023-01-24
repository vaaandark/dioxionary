#!/bin/bash

word="$1"

for i in langdao-ec-gb oxford-gb cdict-gb kdic-computer-gb; do
  if rmall lookup -l "$HOME/.config/rmall/$i" "$word"; then
    exit 0
  fi
done

rmall lookup "$word"
