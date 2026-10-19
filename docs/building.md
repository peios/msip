# Building MSIP

Build on x86-64 Linux with a C/C++ compiler, linker, Python 3.11 or later,
and network access for initial dependency acquisition. The launcher verifies
Bazel 8.4.2 against `build/bazel.sha256`; MODULE.bazel pins Rust 1.98.1 and
rule versions. Cargo.lock pins Rust dependencies.

Run the library, socket, protocol, and package-writer tests:

```sh
./bazel test //...
./bazel build //:packages //:api-docs
```

The `//:msip` target is the Rust library. The source package is in
`bazel-bin/packages.store`, including canonical manifests, compressed blobs,
provenance, and an index. The single package is `dev.peios.msip-source`,
version `0.1.1-1`, architecture `noarch`. It installs the declared source
view under `/usr/src/msip`. It contains neither a daemon nor a C ABI library.

The PPK writer in `third_party/ppk` is an unchanged, pinned source snapshot.
The packaging action verifies its manifest identity and decompresses every
blob to verify its SHA-256 identity. Output is unsigned development material,
not a release. Package versions remain explicit decisions.

## Cargo and style

Cargo remains available to Rust consumers and for diagnostics:

```sh
cargo +1.98.1 test --locked
cargo +1.98.1 clippy --locked --all-targets -- -D warnings
```

Prepare the pinned PSC tools outside the source tree, then run all host checks:

```sh
python3 tools/prepare-style-tools.py /tmp/msip-style
PEIOS_STYLE_BIN=/tmp/msip-style/bin tools/check
```

Use Rust 1.98.1 with rustfmt and Clippy, Node.js 22, and Python with `venv`
support for tooling setup. The style checker excludes the unchanged upstream
PPK source snapshot. Its local Bazel definitions remain maintained here.
Imports preserve their declared order; group standard-library, third-party,
and project-local imports manually when editing.

## Native Peios builds

`--config=peios` selects the installed Peios Rust compiler and hardened
standard library, with control-flow protection and linker hardening. Set
`PEIOS_RUSTC` to the patched compiler when it isn't `/usr/bin/rustc`.
An ordinary Debian compiler is not a replacement for this toolchain.

Acquire inputs on a networked development host:

```sh
tools/offline-build acquire /tmp/msip-inputs
```

Copy the unchanged source tree and snapshot into an isolated native build
environment. It needs Python, Bazel's host requirements, the Peios Rust
compiler, and a compatible C/C++ compiler and linker. Run there:

```sh
tools/offline-build test /tmp/msip-inputs /tmp/msip-work
```

The helper checks the module lock, freezes external repositories, and disables
repository downloads. The environment owner supplies isolation and blocks
network access; the helper doesn't create a container or chroot. Source
packages do not vendor Cargo dependencies. The snapshot supplies those inputs
for offline Bazel builds and is separate from the installed source package.

## Documentation

Use the Trail revision pinned by `.github/workflows/check.yml`. Check the
component documentation and then the full publication context:

```sh
tools/check-docs
tools/check-docs --context ../panorama/docs
```

The component check permits only the exact external-link diagnostic baseline
in `docs/external-links.json`. Publication-context checking resolves links
against the combined tree. Other PGSS chapters remain in Learn staging; the
migration audit records the separate staging-context check.
