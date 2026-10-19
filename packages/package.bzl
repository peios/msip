"""Canonical source packaging and tests for the MSIP library."""

def _source_impl(ctx):
    output = ctx.actions.declare_directory(ctx.label.name + ".store")
    manifest = ctx.actions.declare_file(ctx.label.name + ".inputs.json")
    ctx.actions.write(manifest, json.encode([
        {"path": source.path, "relative": source.short_path}
        for source in ctx.files.srcs
    ]))
    ctx.actions.run(
        executable = "/usr/bin/python3",
        arguments = [
            ctx.file._driver.path,
            "--manifest",
            manifest.path,
            "--output",
            output.path,
            "--writer",
            ctx.executable._writer.path,
            "--checker",
            ctx.executable._checker.path,
            "--version",
            ctx.attr.version,
        ],
        inputs = depset(ctx.files.srcs + [manifest, ctx.file._driver]),
        tools = [ctx.executable._writer, ctx.executable._checker],
        outputs = [output],
        env = {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"},
        execution_requirements = {"no-remote-exec": "1"},
        mnemonic = "PpkSource",
    )
    return [DefaultInfo(files = depset([output]))]

source_package = rule(
    implementation = _source_impl,
    doc = "Package only declared repository sources; verify every canonical blob.",
    attrs = {
        "srcs": attr.label_list(allow_files = True, mandatory = True, doc = "Source files."),
        "version": attr.string(mandatory = True, doc = "Source package revision."),
        "_driver": attr.label(default = "//packages:source.py", allow_single_file = True),
        "_writer": attr.label(default = "//third_party/ppk:ppk", executable = True, cfg = "exec"),
        "_checker": attr.label(default = "//packages:check-blobs", executable = True, cfg = "exec"),
    },
)

def _store_test_impl(ctx):
    executable = ctx.actions.declare_file(ctx.label.name + ".sh")
    ctx.actions.write(executable, """#!/bin/bash
set -euo pipefail
root="${RUNFILES_DIR:-$0.runfiles}/%s"
export PPK_WRITER="$root/%s"
export PPK_CHECKER="$root/%s"
exec /usr/bin/python3 "$root/%s"
""" % (
        ctx.workspace_name,
        ctx.executable._writer.short_path,
        ctx.executable._checker.short_path,
        ctx.file.main.short_path,
    ), is_executable = True)
    return [DefaultInfo(executable = executable, runfiles = ctx.runfiles(
        files = ctx.files.srcs + [ctx.file.main, ctx.executable._writer, ctx.executable._checker],
    ))]

package_test = rule(
    implementation = _store_test_impl,
    test = True,
    doc = "Run Python package-store checks with the declared writer and blob checker binaries.",
    attrs = {
        "main": attr.label(
            allow_single_file = True,
            mandatory = True,
            doc = "Python test entry point.",
        ),
        "srcs": attr.label_list(
            allow_files = True,
            doc = "Declared files available to the action.",
        ),
        "_writer": attr.label(default = "//third_party/ppk:ppk", executable = True, cfg = "exec"),
        "_checker": attr.label(default = "//packages:check-blobs", executable = True, cfg = "exec"),
    },
)
