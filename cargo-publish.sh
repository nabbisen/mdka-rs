#!/bin/sh
#
# BREAK-GLASS ONLY (RFC 015). Do not run this as the normal release path.
#
# crates.io publishing is normally automated and guarded by
# .github/workflows/release-crates.yaml: it runs on `release: created`,
# requires the released commit's CI to have concluded `success` (verify-ci),
# and authenticates via OIDC trusted publishing through the crates-io
# GitHub Environment.
#
# This script bypasses the guarded workflow but still enforces the one
# check it performs: CI must have concluded `success` on the exact commit
# being released. Use it only if release-crates.yaml itself is broken and a
# release must ship regardless. If you find yourself reaching for this
# under normal circumstances, that is a sign the guarded workflow needs
# fixing, not a reason to route around it.

sha=$(git rev-parse HEAD)
conclusion=$(gh run list --commit "$sha" --workflow ci.yaml --limit 1 \
               --json conclusion --jq '.[0].conclusion // empty')
[ "$conclusion" = "success" ] || {
    printf 'CI is not green on %s (got: %s). Refusing to publish.\n' \
           "$sha" "${conclusion:-no run found}" >&2
    exit 1
}

cargo publish --workspace
