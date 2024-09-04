#!/usr/bin/env bash
#
# test locally running instance
#

set -o errexit
set -o pipefail
set -o nounset

curl \
	--data 'callback=cb&regex=^([a-z]*)$&replacement=X&input=ab&input=ab1' \
	--request POST \
	--verbose \
	http://localhost:4000/test.json
