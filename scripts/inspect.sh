#!/bin/sh

OD="$(command -v od)"
[ -z "$OD" ] && echo "Please install \`od'." && exit 1

"$OD" "$1" -x --endian=big 
