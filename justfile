# Local Variables:
# mode: makefile
# End:
# vim: set ft=make :

gen-notes:
  cd arhiv-modules; cargo run --bin generate-notes

remove-arhiv:
  cd arhiv; cargo run --bin remove-arhiv

init-arhiv:
  cd arhiv; cargo run --bin arhiv init prime

arhiv *PARAMS:
  cd arhiv; cargo run --bin arhiv {{PARAMS}}

reset-arhiv: remove-arhiv init-arhiv

prod-build-install:
  cd {{invocation_directory()}}; makepkg -efi; rm -rf pkg

check-ts:
  yarn compiler-errors

lint-ts:
  yarn lint

test-ts:
  yarn tester

validate-ts: check-ts lint-ts test-ts

ui-start-web:
  cd arhiv-ui; yarn start

ui-start-shell:
  cd arhiv-ui; cargo run --features app-shell/dev-server

check:
  cargo check

web-platform:
 cd ts-web-platform; yarn start
