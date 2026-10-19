#!/usr/bin/python3
"""Package the declared MSIP source view with the canonical PPK writer."""

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import tempfile
import tomllib
from pathlib import Path

NAME = "dev.peios.msip-source"
WRITER_REVISION = "9d586e59b6560a2e95eb41564fdcc759ea4f0733"


def digest(path: Path) -> str:
    """Return a file's SHA-256 identity."""
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def materialize(manifest: Path, destination: Path) -> dict[str, str]:
    """Copy declared files, rejecting duplicate or escaping installation paths."""
    fingerprints = {}
    for entry in json.loads(manifest.read_text(encoding="utf-8")):
        name = Path(entry["relative"])
        if name.is_absolute() or ".." in name.parts or not name.parts:
            raise ValueError(f"unsafe source path: {name}")
        if str(name) in fingerprints:
            raise ValueError(f"duplicate source path: {name}")
        source = Path(entry["path"])
        if not source.is_file():
            raise ValueError(f"source is not a regular file: {source}")
        target = destination / name
        target.parent.mkdir(parents=True, exist_ok=True)
        # Bazel input symlinks are execution plumbing, not installed symlinks.
        shutil.copyfile(source, target)
        mode = 0o755 if source.stat().st_mode & 0o111 else 0o644
        target.chmod(mode)
        fingerprints[str(name)] = digest(target) + ":" + oct(mode)
    if not fingerprints:
        raise ValueError("empty source package")
    return fingerprints


def commit(manifest: Path, output: Path, writer: Path, checker: Path, version: str) -> None:
    """Write and verify a source package; the result is unsigned development output."""
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._+~-]*", version):
        raise ValueError("invalid package version")
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".payload-", dir=output) as temporary:
        payload = Path(temporary)
        fingerprints = materialize(manifest, payload / "usr/src/msip")
        identity = subprocess.check_output(
            [
                str(writer.resolve()),
                "commit",
                NAME,
                version,
                "noarch",
                "--content",
                str(payload),
                "--output",
                str(output.resolve()),
            ],
            text=True,
        ).strip()
    relative = Path("manifests") / NAME / version / "noarch/manifest.toml"
    if identity != "sha256:" + digest(output / relative):
        raise ValueError("writer returned an incorrect manifest identity")
    document = tomllib.loads((output / relative).read_text(encoding="utf-8"))
    blobs = output / "blobs/sha256"
    expected = {
        entry["blob"][7:] + ".zst" for entry in document["files"].values() if "blob" in entry
    }
    if {path.name for path in blobs.iterdir()} != expected:
        raise ValueError("missing or unreferenced package blobs")
    subprocess.run([str(checker.resolve()), str(blobs.resolve())], check=True)
    provenance = {
        "name": NAME,
        "version": version,
        "arch": "noarch",
        "manifest": str(relative),
        "manifest_hash": identity,
        "release_ready": False,
        "trust": "development",
        "writer_revision": WRITER_REVISION,
        "sources": fingerprints,
    }
    (output / "provenance.json").write_text(
        json.dumps(provenance, sort_keys=True, indent=2) + "\n", encoding="utf-8"
    )
    (output / "index.json").write_text(
        json.dumps({"packages": [provenance]}, sort_keys=True, indent=2) + "\n", encoding="utf-8"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("manifest", "output", "writer", "checker"):
        parser.add_argument("--" + name, required=True, type=Path)
    parser.add_argument("--version", required=True)
    args = parser.parse_args()
    commit(args.manifest, args.output, args.writer, args.checker, args.version)


if __name__ == "__main__":
    main()
