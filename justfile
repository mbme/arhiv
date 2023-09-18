# vim: set ft=make :

remove-arhiv:
  cd arhiv-tools; cargo run --bin remove-arhiv

init-arhiv:
  cargo run --bin arhiv init test-arhiv

arhiv *PARAMS:
  cargo run --bin arhiv {{PARAMS}}

arhiv-server:
  just arhiv server

reset-arhiv: remove-arhiv init-arhiv

run:
  cd arhiv-ui; yarn run clean; tmux new-session -s arhiv-ui \
     'watchexec -r --debounce=4000 --exts rs -- "notify-send Restarting... -t 2000; RUST_BACKTRACE=1 cargo run -p binutils --bin arhiv server"' \; \
     split-window -h 'yarn run watch:js' \; \
     split-window 'yarn run watch:css' \; \
     select-pane -t 0

scrape *PARAMS:
  cargo run --bin mb-scraper {{PARAMS}}

bump-version:
  ./bump-version.sh

prod-build-install:
  cp PKGBUILD.template PKGBUILD
  makepkg -efi || true
  rm -rf pkg
  rm -f *.pkg.tar.zst
  rm PKGBUILD
  systemctl --user daemon-reload
  systemctl --user restart arhiv-ui-server.service

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
  yarn run check

check: check-rs check-ts

clean-all:
  cargo clean
  cargo clean --release
  rm -rf .log
