#!/bin/sh
#
# BREAK-GLASS ONLY (RFC 015). Do not run this as the normal release path.
#
# crates.io publishing is now automated and guarded by
# .github/workflows/release-crates.yaml: it runs on `release: created`,
# requires the released commit's CI to have concluded `success`
# (verify-ci), and requires a human reviewer approval on the `crates-io`
# GitHub Environment before it runs `cargo publish`.
#
# This script bypasses all of that - no CI check, no approval gate, no
# record in a workflow run. Use it only if release-crates.yaml itself is
# broken and a release must ship regardless. If you find yourself reaching
# for this under normal circumstances, that is a sign the guarded workflow
# needs fixing, not a reason to route around it.

cargo package
cargo publish

crates="cli node python"
for crate in $crates; do
    cd $crate
    cargo package
    cargo publish
    cd ..
done
