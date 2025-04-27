# vim: set ft=make :

home := env("HOME")
root := home + "/temp/arhiv"
debug_log_level := "debug,h2=info,rustls=info,i18n_embed=warn,rs_utils::http_server=info,hyper=info,axum::rejection=trace,keyring=info"

# WARN: the --platform MUST match minSdk from build.gradle
android_platform_version := "30"

alias c := check

arhiv *PARAMS:
  npm run build --workspace arhiv
  DEV_ARHIV_ROOT="{{root}}" SERVER_PORT=8443 cargo run --bin arhiv {{PARAMS}}

run:
  cd arhiv; npm run clean; tmux new-session -s arhiv \
     'DEV_ARHIV_ROOT={{root}} SERVER_PORT=8443 RUST_LOG={{debug_log_level}} BROWSER=chromium cargo run -p binutils --bin arhiv server --browser' \; \
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

print-version:
  @echo $(git describe --abbrev=0 --tags --always)

_print-long-version:
  @echo $(git describe --tags --always)

bump-version:
  #!/usr/bin/env bash
  CURRENT_VERSION=$(just print-version)
  NEXT_VERSION=$(($CURRENT_VERSION + 1))

  echo "Current version is $CURRENT_VERSION"
  echo "Next version is $NEXT_VERSION"

  read -p "Are you sure you want to create new tag $NEXT_VERSION? (y)" -n 1 -r
  echo    # (optional) move to a new line
  if [[ $REPLY =~ ^[Yy]$ ]]
  then
    git tag $NEXT_VERSION
    git push
    git push --tags
  else
    echo "Cancelled"
  fi

arch-install:
  cp PKGBUILD.template PKGBUILD
  makepkg -efi || true
  rm -rf pkg
  rm -f *.pkg.tar.zst
  rm PKGBUILD

_prod-npm-build:
  npm run prod:build --workspace arhiv

prod-build: _prod-npm-build
  TYPED_V_VERSION=$(just _print-long-version) \
  cargo build --frozen --release --features production-mode -p binutils --bin arhiv

prod-build-desktop:
  npm run prod:build --workspace arhiv-desktop

# install the Arhiv CLI locally using Cargo
cargo-install: _prod-npm-build
  cargo install --path binutils --bin arhiv --features production-mode

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

_prepare_to_building_android_libs: _prod-npm-build
  rm -rf arhiv-android/app/src/main/jniLibs
  mkdir arhiv-android/app/src/main/jniLibs

build-android-libs: _prepare_to_building_android_libs
  cd arhiv-android; \
  cargo ndk -t x86_64 -t arm64-v8a --platform {{android_platform_version}} -o ./app/src/main/jniLibs build --release

prod-build-android-libs: _prepare_to_building_android_libs
  cd arhiv-android; \
  TYPED_V_VERSION=$(just _print-long-version) \
  cargo ndk -t arm64-v8a --platform {{android_platform_version}} -o ./app/src/main/jniLibs build --frozen --release --features production-mode

build-android-app:
  cd arhiv-android; ./gradlew assembleDebug

prod-build-android-app:
  cd arhiv-android; ./gradlew assembleRelease --no-daemon
  mv ./arhiv-android/app/build/outputs/apk/release/app-release-unsigned.apk arhiv.apk

install-android-app:
  cd arhiv-android; ./gradlew installDebug

# ----------------------------------------------------

bench *PARAMS:
  cd rs-utils; cargo bench -- {{PARAMS}}

profile-benchmark:
  cargo flamegraph --dev --root --bench container_benchmark -- --bench

build-timings:
  cd {{invocation_directory()}}; cargo +nightly build -Ztimings

clear-timings:
  cd {{invocation_directory()}}; rm cargo-timing*
