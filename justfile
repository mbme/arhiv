# Local Variables:
# mode: makefile
# End:
# vim: set ft=make :

generate-fakes:
  cd arhiv; cargo run --bin generate-fakes

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

test-ts *PARAMS:
  yarn tester {{PARAMS}}

validate-ts: check-ts lint-ts test-ts

ui-start-web:
  cd arhiv-ui; watchexec --exts ts,tsx yarn build

ui-start-shell-server:
  cd arhiv-ui; RUST_LOG=INFO cargo run --features app-shell/dev-server

ui-start-shell:
  cd arhiv-ui; RUST_LOG=INFO cargo run

check:
  cargo check

web-platform:
 cd ts-web-platform; yarn start
