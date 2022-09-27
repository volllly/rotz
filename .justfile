set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

alias r := run

alias t := test

alias n := nextest

alias c := clippy

alias d := run-docs


bt := "0"

export RUST_BACKTRACE := bt

log := "warn"

export JUST_LOG := log

test *pattern:
  cargo test {{pattern}} --all-features

nextest *pattern:
  cargo nextest run {{pattern}} --all-features

run:
  cargo run

build:
  cargo build

fmt:
  cargo +nightly fmt

clippy:
  cargo clippy --all-features

loc:
  tokei

run-docs:
  cd docs && npm start

update-src:
  cargo update

update-docs:
  cd docs && npm update

update: update-src update-docs

clean-src:
  cargo clean

clean-docs:
  cd docs && npm run clear

clean: clean-src clean-docs