#!/bin/sh

cargo package
cargo publish

crates="cli node python"
for crate in $crates; do
    cd $crate
    cargo package
    cargo publish
    cd ..
done
