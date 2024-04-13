# vim: set ft=make :

dev_cert_nickname := "arhiv-dev"
home := env("HOME")
root := home + "/temp/arhiv"
dev_cert_path := root + "/certificate.pfx"

arhiv *PARAMS:
  DEV_ARHIV_ROOT="{{root}}" cargo run --bin arhiv {{PARAMS}}

arhiv-server:
  just arhiv server

run:
  cd arhiv; npm run clean; tmux new-session -s arhiv \
     'DEV_ARHIV_ROOT={{root}} RUST_LOG=debug,h2=info,rustls=info,mdns_sd=info,rs_utils=info,hyper=info,axum::rejection=trace cargo run -p binutils --bin arhiv server --port 8443' \; \
     split-window -h 'npm run watch:js' \; \
     split-window 'npm run watch:css' \; \
     select-pane -t 0

update-browser-certificates:
  certutil -d sql:$HOME/.pki/nssdb -D -n "{{dev_cert_nickname}}" || true
  pk12util -d sql:$HOME/.pki/nssdb -i "{{dev_cert_path}}"

desktop *ARGS:
  npm run build --workspace arhiv-desktop
  npm run start --workspace arhiv-desktop {{ARGS}}

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

# install the arhiv locally using Cargo
cargo-install:
  npm install
  cargo install --path binutils --bin arhiv --bin mb-scraper --features production-mode

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
