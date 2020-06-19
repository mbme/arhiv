# Local Variables:
# mode: makefile
# End:
# vim: set ft=make :

export RUST_LOG := "DEBUG"

gen-notes:
  cd arhiv-notes && cargo run --bin generate-notes

remove-arhiv:
  cd arhiv && cargo run --bin remove-arhiv

init-arhiv:
  cd arhiv && cargo run --bin arhiv init

reset-arhiv: remove-arhiv init-arhiv
