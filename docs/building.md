# Building MSIP

Use pekit to build the MSIP library and run its tests. Pekit builds each
crate on its own, so a change rebuilds only the crates it reaches. The
supported development host is x86-64 Linux.

## Before you start

You need pekit and Git. Everything else is pinned and fetched by pekit on
the first build: the Bootlin C toolchain, Rust 1.98.1 with Clippy, Python
3.14, and the crates `Cargo.lock` selects. The first build needs network
access; later builds find what they need in pekit's store, and every step
runs without network access in a sandbox that holds only its declared
inputs.

The build files use the gcslib libraries named in `workspace.gcsl` and the
build files. `pekit lock` pins their commits in `pekit.lock`.

## Build

```sh
pekit build api_docs
```

| Target | Output |
|---|---|
| `api_docs` | The library's API documentation, made by rustdoc with every warning denied |

The library itself is a Rust crate that consumers build from source. It
has no C ABI, shared library, or daemon program.

## Test

```sh
pekit test
```

| Test | Check |
|---|---|
| `msip` | The unit tests and the conversation, framing, loopback, state, and validation tests |
| `doctests` | The documentation tests |
| `docs` | rustdoc's checks of the API documentation, with every warning denied |
| `clippy` | Clippy's checks of every target, with every warning denied |

A test whose inputs did not change is found in the store and not run
again.

For the full contributor check, install the pinned PSC tools into a
directory outside the repository, then run the shared entry point:

```sh
python3 tools/prepare-style-tools.py "$HOME/.cache/msip-style"
export PEIOS_STYLE_BIN="$HOME/.cache/msip-style/bin"
tools/check
```

`tools/check` checks owned source formatting and lint with rustfmt, Ruff,
ShellCheck, shfmt, and Prettier, then runs `pekit test`. The style checks
use the host's Rust 1.98.1 formatter through rustup. The tool versions and
hashes are recorded in `tools/style-tools.lock.json`. GitHub's workflow
runs the style checks alone; builds and tests run with pekit.

Imports preserve their declared order; group standard-library, third-party,
and project-local imports manually when editing.

## How the build is planned

`build.gcsl` plans the Cargo workspace with the gcslib rust library's
`workspace`: Cargo works out the build, unit by unit, and pekit compiles
each unit as a step. Planning reads the manifests, `Cargo.lock`, and which
files exist, never the sources, so editing a source plans nothing again.
Keep Rust dependency versions in `Cargo.toml` and `Cargo.lock`, and update
the lockfile deliberately with Cargo.

Cargo remains available to Rust consumers and for diagnostics:

```sh
cargo +1.98.1 test --locked
cargo +1.98.1 clippy --locked --all-targets -- -D warnings
```

## Not built with pekit yet

The source package, `dev.peios.msip-source`, and builds with the native
Peios toolchain, which hardens programs with control-flow enforcement
technology (CET), are not yet built with pekit. The package definition and
native build settings from the Bazel build are kept outside this
repository until they are.

## Documentation

Check the component documentation and then the full publication context:

```sh
tools/check-docs
tools/check-docs --context ../learn
```

Set `TRAIL` to the Trail executable if it is not on `PATH`. The component
check permits only the exact external-link diagnostic baseline in
`docs/external-links.json`. Publication-context checking resolves links
against the combined tree. The normative MSIP chapter lives centrally in
Learn, in the Peios Specifications product, alongside the other PGSS
chapters.
