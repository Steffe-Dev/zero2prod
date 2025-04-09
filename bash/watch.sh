#!/usr/bin/env bash
cargo watch\
	-x "fix --allow-dirty"\
	-x check\
	-x test\
	-x "clippy -- -D warnings"\
	-x run
