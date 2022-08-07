# vim: set ft=make :

generate-fakes:
  cd arhiv-tools; cargo run --bin generate-fakes

remove-arhiv:
  cd arhiv-tools; cargo run --bin remove-arhiv

init-arhiv:
  cd arhiv-bin; cargo run --bin arhiv init test-arhiv --prime

arhiv *PARAMS:
  cd arhiv-bin; cargo run --bin arhiv {{PARAMS}}

arhiv-server:
  just arhiv prime-server

reset-arhiv: remove-arhiv init-arhiv

web3:
  cd arhiv-ui3; tmux new-session -s arhiv-ui3 \
     'watchexec -r -d 4000 --exts rs -- "notify-send Restarting... -t 2000; RUST_BACKTRACE=1 cargo run"' \; \
     split-window -h 'yarn watch:js' \; \
     split-window 'yarn watch:css' \; \
     select-pane -t 0

bump-version:
  ./bump-version.sh

prod-build-install:
  cp PKGBUILD.template PKGBUILD
  makepkg -efi || true
  rm -rf pkg
  rm -f *.pkg.tar.zst
  rm PKGBUILD

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
  yarn run check-fmt
  yarn workspace arhiv-ui3 run lint
  yarn workspace arhiv-ui3 run compiler-errors
  yarn workspace scraper run lint
  yarn workspace scraper run compiler-errors

check: check-rs check-ts

clean-all:
  cargo clean
  cargo clean --release
  rm -rf .log
