# MSIP

MSIP implements the Multi-Surface Interaction Protocol in Rust. A daemon
owns a conversation; terminal, graphical, and automated surfaces render its
turns and submit answers. The library supplies framing, message types,
answer checks, and shared page state. The embedding application owns
transport, authentication, access control, binding, and domain validation.

Read the [protocol specification](https://learn.peios.org/specifications/pgss/msip/scope-and-roles),
the [library documentation](docs/learn/), the
[build guide](docs/building.md), and the [compatibility notes](docs/compatibility.md).
The public documentation merges into Peios Learn as part of the Peios product.

Build and test on x86-64 Linux:

```sh
./bazel test //...
./bazel build //:packages //:api-docs
```

Cargo remains supported for Rust consumers. Pin an exact source revision;
this repository does not publish a C ABI, shared library, or daemon binary.
The source package is unsigned development output. See [LICENSE](LICENSE).
