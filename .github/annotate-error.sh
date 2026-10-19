#!/usr/bin/env bash
# Prints stdin as one error annotation titled $1: a run's logs need a GitHub sign-in, its annotations don't.
awk -v title="$1" 'BEGIN { ORS = "%0A"; printf "::error title=%s::", title }
  { gsub(/\r/, ""); gsub(/%/, "%25"); print substr($0, 1, 500) }'
echo
