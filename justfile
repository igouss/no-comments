set allow-duplicate-variables := true

import 'scripts/mutation-triage/mutation-triage.just'

mt_green_cmd := "cargo nextest run --all-features"
mt_test_env := "CI=1"

default: check

[doc("Every gate: format, clippy, ledger integrity, tests, and the tool on its own tree.")]
check: fmt-check clippy mutants-gates test self-scan

fmt-check:
    cargo fmt --all --check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo nextest run --all-features

[doc("Run no-comments on this repository. It is its own first consumer.")]
self-scan:
    cargo run --quiet -- .

[doc("Put no-comments on PATH.")]
install:
    cargo install --locked --path .
