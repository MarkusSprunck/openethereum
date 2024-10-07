#!/bin/sh

# Check to see if a pipe exists on stdin.
if [ -p /dev/stdin ]; then
        # We want to read the input line by line
        while IFS= read line; do
              array=${line}
              time=$( echo $array | cut -d' ' -f 2)
              date=$( echo $array | cut -d' ' -f 1)
              message=$( echo $array | cut  -b25- | sed 's/"/\\"/g')
              # write result to console
              echo "{\"message\":\"$message\"}"
        done
fi