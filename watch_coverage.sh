#!/usr/bin/sh

# Configured to work with VSCode and the Coverage Gutters extension
cargo watch -x 'llvm-cov nextest --lcov --output-path ./.analyze/lcov.info' -w src
