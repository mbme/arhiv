# Installation
* `just prod-build-install`
* `sudo systemctl daemon-reload`
* `sudo systemctl enable --now arhiv-service@<username>`

# Build dependencies
* `rust`
* `cargo`
* `just` command runner https://github.com/casey/just
* `nodejs`
* `npm`

# Dev dependencies
* `watchexec` to run commands in response to file modifications
* `cargo-insta` to manage snapshot tests
* `tmux` for running dev servers in parallel

## Projects
* [binutils](binutils) - cli apps for controling laptop backlight, volume, microphone, touchpad etc.
* [utils](rs-utils) - various helpers

## Special switches
* `JSON_ARG_MOODE` env variable for some CLIs allows to receive arguments as a JSON object
* `production-mode` feature flag