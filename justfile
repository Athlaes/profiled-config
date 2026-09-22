[windows]
set shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

build:
    cargo build --workspace --all-targets --all-features --locked

test:
    cargo test --workspace --all-features --locked

coverage:
    cargo llvm-cov --workspace --all-features --locked --lcov --output-path lcov.info

check-coverage: coverage
    cargo llvm-cov report --locked --fail-under-lines 80

clippy:
    cargo clippy --workspace --all-targets --all-features --locked

upgrade:
    cargo upgrade

upgrade-incompatible:
    cargo upgrade --incompatible
