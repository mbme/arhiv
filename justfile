# vim: set ft=make :

home := env("HOME")
root := home + "/temp/arhiv"
debug_log_level := "debug,h2=info,rustls=info,i18n_embed=warn,rs_utils=info,hyper=info,axum::rejection=trace"

alias c := check

arhiv *PARAMS:
  DEV_ARHIV_ROOT="{{root}}" cargo run --bin arhiv {{PARAMS}}

arhiv-server:
  just arhiv server

run:
  cd arhiv; npm run clean; tmux new-session -s arhiv \
     'DEV_ARHIV_ROOT={{root}} SERVER_PORT=8443 RUST_LOG={{debug_log_level}} cargo run -p binutils --bin arhiv server' \; \
     split-window -h 'npm run watch:js' \; \
     split-window 'npm run watch:css' \; \
     select-pane -t 0

desktop *ARGS:
  #!/usr/bin/env bash
  set -euxo pipefail

  npm run build --workspace arhiv
  npm run build --workspace arhiv-desktop

  export DEV_ARHIV_ROOT={{root}}
  export SERVER_PORT=8443
  export RUST_LOG={{debug_log_level}}

  cargo build -p binutils

  export ARHIV_BIN="{{justfile_directory()}}/target/debug/arhiv"
  export ELECTRON_OZONE_PLATFORM_HINT=wayland

  npm run start --workspace arhiv-desktop -- {{ARGS}}

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
  npm run prod:build --workspace arhiv
  cargo install --path binutils --bin arhiv --features production-mode

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

build-android-libs *RELEASE_FLAG:
  #!/usr/bin/env bash
  set -euxo pipefail

  npm run prod:build --workspace arhiv

  cd arhiv-android

  rm -rf ./app/src/main/jniLibs
  mkdir ./app/src/main/jniLibs

  # WARN: the --platform MUST match minSdk from build.gradle
  cargo ndk -t x86_64 -t arm64-v8a --platform 30 -o ./app/src/main/jniLibs build {{RELEASE_FLAG}}

prod-build-android-libs:
  just build-android-libs --release --features production-mode

build-android-app: build-android-libs
  #!/usr/bin/env bash
  set -euxo pipefail

  cd arhiv-android

  ./gradlew assembleDebug

bench *PARAMS:
  cd rs-utils; cargo bench -- {{PARAMS}}

profile-benchmark:
  cargo flamegraph --dev --root --bench container_benchmark -- --bench
