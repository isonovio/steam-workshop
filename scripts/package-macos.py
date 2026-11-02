#!/usr/bin/env python3
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Self

ROOT = Path(__file__).resolve().parents[1]


def cargo(*args: str) -> str:
    result = subprocess.run(
        ["mise", "exec", "--", "cargo", *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.stderr:
        print(result.stderr, file=sys.stderr, end="")
    if result.returncode:
        print(result.stdout, file=sys.stderr)
        result.check_returncode()
    return result.stdout


@dataclass(frozen=True)
class Release:
    version: str
    executable: Path
    library: Path
    output: Path

    @classmethod
    def build(cls) -> Self:
        metadata = json.loads(cargo("metadata", "--locked", "--format-version=1"))
        package = next(p for p in metadata["packages"] if p["name"] == "steam-workshop")
        native = next(p for p in metadata["packages"] if p["name"] == "steamworks-sys")
        messages = [
            json.loads(line) for line in cargo("build", "--release", "--locked", "--message-format=json").splitlines()
        ]
        executable = next(
            m["executable"]
            for m in messages
            if m["reason"] == "compiler-artifact" and m.get("executable") and m["package_id"] == package["id"]
        )
        native_output = next(
            m["out_dir"] for m in messages if m["reason"] == "build-script-executed" and m["package_id"] == native["id"]
        )
        return cls(
            version=package["version"],
            executable=Path(executable),
            library=Path(native_output) / "libsteam_api.dylib",
            output=Path(metadata["target_directory"]) / "dist",
        )

    def stage(self, directory: Path) -> None:
        shutil.copy2(self.executable, directory / "steam-workshop")
        library = directory / self.library.name
        shutil.copy2(self.library, library)
        library.chmod(0o644)

    def package(self) -> Path:
        self.output.mkdir(exist_ok=True)
        archive = self.output / f"steam-workshop-v{self.version}-aarch64-apple-darwin.tar.gz"
        with tempfile.TemporaryDirectory(prefix="steam-workshop-release-") as temporary:
            directory = Path(temporary)
            self.stage(directory)
            verify_executable(directory)
            with tarfile.open(archive, "w:gz") as bundle:
                for file in sorted(directory.iterdir()):
                    bundle.add(file, arcname=file.name)
        write_checksum(archive)
        return archive


def verify_executable(directory: Path) -> None:
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith(("DYLD_", "LD_LIBRARY_PATH", "Steam", "STEAM_WORKSHOP_CHILD_"))
    }
    subprocess.run([str(directory / "steam-workshop"), "--version"], cwd=directory, env=environment, check=True)


def write_checksum(archive: Path) -> None:
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_name(f"{archive.name}.sha256").write_text(f"{digest}  {archive.name}\n")


def main() -> None:
    assert (platform.system(), platform.machine()) == ("Darwin", "arm64"), "This release recipe supports macOS ARM64"
    print(Release.build().package())


if __name__ == "__main__":
    main()
