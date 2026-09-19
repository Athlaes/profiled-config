[windows]
set shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

test:
    cargo test --workspace --all-features --locked
clippy:
    cargo clippy --workspace --all-targets --all-features --locked
