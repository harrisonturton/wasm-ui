#!/bin/bash

# This is the Git precommit hook script. It runs before any changes are
# committed. It is symlinked as `.git/hooks/precommit`.

cd core
cargo fmt
