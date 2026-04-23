#!/bin/sh

cargo package
cargo publish

crates="cli node python"
for crate in $crates; do
    cargo package
    cargo publish
done
