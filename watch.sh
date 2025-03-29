#!/usr/bin/env bash
cargo watch -x check -x test -x "clippy -- -D warnings" -x run
