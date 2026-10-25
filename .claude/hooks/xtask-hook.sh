#!/bin/sh
# Runs `xtask hook <event>` for Claude Code (see xtask/src/hook.rs).
#
# Calls the built xtask binary directly when it is up to date: `cargo run`
# would wait for the build-directory lock whenever a test run holds it, and
# every file edit would stall behind it. A stale or missing binary is rebuilt
# through cargo.
cd "${CLAUDE_PROJECT_DIR:-$(dirname "$0")/../..}" || exit 0
bin=target/debug/xtask
for src in xtask/src/*.rs xtask/Cargo.toml; do
    if [ ! -x "$bin" ] || [ "$src" -nt "$bin" ]; then
        exec cargo xtask hook "$@"
    fi
done
exec "$bin" hook "$@"
