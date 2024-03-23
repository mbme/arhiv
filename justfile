# vim: set ft=make :

dev_cert_nickname := "arhiv-dev-server"
dev_cert_path := justfile_directory() + "/certificate.pfx"

arhiv *PARAMS:
  DEBUG_ARHIV_ROOT=~/temp/arhiv cargo run --bin arhiv {{PARAMS}}

arhiv-server:
  just arhiv server

run:
  cd arhiv; npm run clean; tmux new-session -s arhiv \
     'DEBUG_ARHIV_ROOT=~/temp/arhiv ARHIV_SERVER_CERTIFICATE="{{dev_cert_path}}" RUST_BACKTRACE=1 RUST_LOG=debug,h2=info,rustls=info,mdns_sd=info,rs_utils=info,hyper=info,axum::rejection=trace cargo run -p binutils --bin arhiv server' \; \
     split-window -h 'npm run watch:js' \; \
     split-window 'npm run watch:css' \; \
     select-pane -t 0

export-certificate:
  rm {{dev_cert_path}} 2> /dev/null || true
  just arhiv export-certificate "{{dev_cert_path}}" --name "{{dev_cert_nickname}}"

update-browser-certificates:
  certutil -d sql:$HOME/.pki/nssdb -D -n "{{dev_cert_nickname}}" || true
  pk12util -d sql:$HOME/.pki/nssdb -i "{{dev_cert_path}}"

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

build-android-libs:
  cd arhiv-android; rm -rf ./app/src/main/jniLibs; ANDROID_NDK_HOME=~/Android/Sdk/ndk/ cargo ndk -t x86_64 -t arm64-v8a -o ./app/src/main/jniLibs build # --release
