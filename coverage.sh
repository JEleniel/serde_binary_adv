#!/usr/bin/sh

# Configured to work with VSCode and the Coverage Gutters extension
cargo llvm-cov nextest --lcov --output-path ./.analyze/lcov.info
