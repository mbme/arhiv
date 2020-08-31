# Local Variables:
# mode: makefile
# End:
# vim: set ft=make :

gen-notes:
  cd arhiv-ui && cargo run --bin generate-notes

remove-arhiv:
  cd arhiv && cargo run --bin remove-arhiv

init-arhiv:
  cd arhiv && cargo run --bin arhiv init-prime

arhiv *PARAMS:
  cd arhiv && cargo run --bin arhiv {{PARAMS}}

reset-arhiv: remove-arhiv init-arhiv

check-ts:
  yarn compiler-errors

lint-ts:
  yarn lint

test-ts:
  yarn test

validate-ts: check-ts lint-ts test-ts
