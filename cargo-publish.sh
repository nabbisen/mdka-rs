#!/bin/sh
#
# crates.io release path (RFC 015 amendment). Run locally by the maintainer.
#
# crates.io publishing is deliberately manual and is not covered by the
# verify-ci guard that gates release-executable.yaml, release-npm.yaml, and
# release-pypi.yaml. This script enforces the one check that guard used to
# perform: CI must have concluded `success` on the exact commit being
# released.

sha=$(git rev-parse HEAD)
conclusion=$(gh run list --commit "$sha" --workflow ci.yaml --limit 1 \
               --json conclusion --jq '.[0].conclusion // empty')
[ "$conclusion" = "success" ] || {
    printf 'CI is not green on %s (got: %s). Refusing to publish.\n' \
           "$sha" "${conclusion:-no run found}" >&2
    exit 1
}

cargo publish --workspace
