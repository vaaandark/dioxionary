#!/bin/bash

# the final result:
#   498 SUCCESS
#   0 FAILURE
#   100% passed

success=0
failure=0

for i in $(rmall list); do
  if rmall lookup -L -l ~/.config/rmall/cdict-gb/ "$i"; then
    ((success++))
  else
    ((failure++))
    echo "cannot find the word $i!"
  fi
done

echo "$success SUCCESS"
echo "$failure FAILURE"
echo "$(( 100 * success / (success + failure) ))% passed"
