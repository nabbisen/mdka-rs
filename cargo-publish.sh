#!/bin/sh
#
# crates.io release path (RFC 015 amendment). Run locally by the maintainer.
#
# crates.io publishing is deliberately manual and is not covered by the
# verify-ci guard that gates release-executable.yaml, release-npm.yaml, and
# release-pypi.yaml. Before running this script, confirm CI is green on the
# commit you are releasing:
#
#   gh run list --branch main --limit 5
#
# Then, from a clean checkout of that commit:

cargo publish --workspace
