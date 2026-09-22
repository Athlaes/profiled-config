[windows]
set shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

test:
    cargo test --workspace --all-features --locked

coverage:
    cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info

clippy:
    cargo clippy --workspace --all-targets --all-features --locked

upgrade:
    cargo upgrade

upgrade-incompatible:
    cargo upgrade --incompatible
