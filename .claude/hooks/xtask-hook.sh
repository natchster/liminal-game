#!/bin/sh
# Runs `xtask hook <event>` for Claude Code (see xtask/src/hook.rs).
#
# Calls the built xtask binary directly when it is up to date: `cargo run`
# would wait for the build-directory lock whenever a test run holds it, and
# every file edit would stall behind it. A stale or missing binary is rebuilt
# first. When the rebuild fails (xtask's sources half edited, perhaps by
# another session), the binary already built still runs, so that the hooks
# go on refusing, formatting and noting edits in the session's ledger.
cd "${CLAUDE_PROJECT_DIR:-$(dirname "$0")/../..}" || exit 0
bin=target/debug/xtask
for src in xtask/src/*.rs xtask/src/*/*.rs xtask/Cargo.toml; do
    if [ ! -x "$bin" ] || [ "$src" -nt "$bin" ]; then
        # cargo's output on stderr: the hook's stdout is its answer
        cargo build --quiet --package xtask >&2 || [ -x "$bin" ] || exit 1
        break
    fi
done
exec "$bin" hook "$@"
