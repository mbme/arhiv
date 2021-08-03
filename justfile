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

bump-minor:
  cd {{invocation_directory()}}; cargo release --no-dev-version minor

bump-patch:
  cd {{invocation_directory()}}; cargo release --no-dev-version patch
