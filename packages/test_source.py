"""Check source mapping and an end-to-end canonical package."""

import json
import os
import tempfile
import tomllib
import unittest
from pathlib import Path

from source import commit, digest, materialize


class SourceTests(unittest.TestCase):
    def test_rejects_escaping_duplicate_and_empty_sources(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "input"
            source.write_text("source", encoding="utf-8")
            manifest = root / "inputs.json"
            for names in [["../escape"], ["/absolute"], ["."], ["same", "same"], []]:
                with self.subTest(names=names):
                    manifest.write_text(
                        json.dumps([{"relative": name, "path": str(source)} for name in names]),
                        encoding="utf-8",
                    )
                    with self.assertRaises(ValueError):
                        materialize(manifest, root / "output")

    def test_canonical_round_trip_and_executable_mode(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            source.write_text("sample source\n", encoding="utf-8")
            script = root / "script"
            script.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            script.chmod(0o755)
            manifest = root / "inputs.json"
            manifest.write_text(
                json.dumps(
                    [
                        {"relative": "src/lib.rs", "path": str(source)},
                        {"relative": "tools/check", "path": str(script)},
                    ]
                ),
                encoding="utf-8",
            )
            writer, checker = Path(os.environ["PPK_WRITER"]), Path(os.environ["PPK_CHECKER"])
            output = root / "store"
            commit(manifest, output, writer, checker, "0.1.1-1")
            info = json.loads((output / "provenance.json").read_text())
            package = tomllib.loads((output / info["manifest"]).read_text())
            self.assertEqual(package["name"], "dev.peios.msip-source")
            self.assertFalse(info["release_ready"])
            self.assertEqual(
                package["files"]["usr/src/msip/src/lib.rs"], {"blob": "sha256:" + digest(source)}
            )
            self.assertTrue(package["files"]["usr/src/msip/tools/check"]["exec"])
            second = root / "second"
            commit(manifest, second, writer, checker, "0.1.1-1")
            self.assertEqual(
                (output / info["manifest"]).read_bytes(), (second / info["manifest"]).read_bytes()
            )


if __name__ == "__main__":
    unittest.main()
