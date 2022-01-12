# Local Variables:
# mode: makefile
# End:
# vim: set ft=make :

generate-fakes:
  cd arhiv-utils; cargo run --bin generate-fakes

remove-arhiv:
  cd arhiv-utils; cargo run --bin remove-arhiv

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

prod-build-install:
  cd {{invocation_directory()}}; cp PKGBUILD.template PKGBUILD; makepkg -efi; rm -rf pkg; rm *.pkg.tar.zst; rm PKGBUILD

arhiv-bump-minor *PARAMS:
  cd arhiv-bin; cargo release --no-dev-version minor {{PARAMS}}

arhiv-bump-patch *PARAMS:
  cd arhiv-bin; cargo release --no-dev-version patch {{PARAMS}}

binutils-bump-minor *PARAMS:
  cd binutils; cargo release --no-dev-version minor {{PARAMS}}

test-scrapers:
  cd arhiv-scraper; cargo test -- --ignored --test-threads 1

build-timings:
  cd {{invocation_directory()}}; cargo +nightly build -Ztimings

clear-timings:
  cd {{invocation_directory()}}; rm cargo-timing*

check:
  cargo clippy --verbose --all-targets --all-features -- -D warnings
