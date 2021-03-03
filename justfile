# Local Variables:
# mode: makefile
# End:
# vim: set ft=make :

generate-fakes:
  cd arhiv-utils; cargo run --bin generate-fakes

remove-arhiv:
  cd arhiv-utils; cargo run --bin remove-arhiv

init-arhiv:
  cd arhiv; cargo run --bin arhiv init

arhiv *PARAMS:
  cd arhiv; cargo run --bin arhiv {{PARAMS}}

arhiv-server:
  just arhiv server

reset-arhiv: remove-arhiv init-arhiv

check-ts:
  yarn compiler-errors

lint-ts:
  yarn lint

test-ts *PARAMS:
  yarn tester {{PARAMS}}

validate-ts: check-ts lint-ts test-ts

ui-start-web:
  cd arhiv-ui; watchexec --exts ts,tsx yarn build

ui-start-shell:
  just arhiv ui --open=chromium

web-platform:
  cd ts-web-platform; yarn start

prod-build-install:
  cd {{invocation_directory()}}; makepkg -efi; rm -rf pkg

prod-install-all:
  cd binutils; just prod-build-install
  cd arhiv; just prod-build-install
  cd arhiv-ui; just prod-build-install
