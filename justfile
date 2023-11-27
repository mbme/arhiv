# vim: set ft=make :

arhiv *PARAMS:
  DEBUG_ARHIV_ROOT=~/temp/arhiv cargo run --bin arhiv {{PARAMS}}

arhiv-server:
  just arhiv server

run:
  cd arhiv; npm run clean; tmux new-session -s arhiv \
     'watchexec -r --debounce=4000 --exts rs -- "notify-send Restarting... -t 2000; DEBUG_ARHIV_ROOT=~/temp/arhiv  RUST_BACKTRACE=1 RUST_LOG=debug,axum=trace,mdns_sd=info,rs_utils=info,hyper=info cargo run -p binutils --bin arhiv server"' \; \
     split-window -h 'npm run watch:js' \; \
     split-window 'npm run watch:css' \; \
     select-pane -t 0

scrape *PARAMS:
  cargo run --bin mb-scraper {{PARAMS}}

mdns-tester:
  cargo run --bin mdns-tester

bump-version:
  ./bump-version.sh

prod-build-install:
  cp PKGBUILD.template PKGBUILD
  makepkg -efi || true
  rm -rf pkg
  rm -f *.pkg.tar.zst
  rm PKGBUILD
  systemctl --user daemon-reload
  systemctl --user restart arhiv-server.service

test-scrapers *PARAMS:
  cd scraper; cargo test -- --ignored --test-threads 1 {{PARAMS}}

build-timings:
  cd {{invocation_directory()}}; cargo +nightly build -Ztimings

clear-timings:
  cd {{invocation_directory()}}; rm cargo-timing*

check-rs:
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test

check-ts:
  npm run check

check: check-rs check-ts

clean-all:
  cargo clean
  cargo clean --release
  rm -rf .log
