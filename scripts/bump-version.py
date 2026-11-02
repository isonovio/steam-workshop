#!/usr/bin/env python3
import os
import re
import shlex
import subprocess
import sys
import tempfile
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Self

ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class Version:
    major: int
    minor: int
    patch: int

    @classmethod
    def parse(cls, value: str) -> Self:
        assert re.fullmatch(r"(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)", value), f"Unsupported version: {value}"
        return cls(*map(int, value.split(".")))

    def bump(self, part: str) -> Self:
        match part:
            case "major":
                return type(self)(self.major + 1, 0, 0)
            case "minor":
                return type(self)(self.major, self.minor + 1, 0)
            case "patch":
                return type(self)(self.major, self.minor, self.patch + 1)
            case _:
                raise AssertionError(f"Unknown version part: {part}")

    def __str__(self) -> str:
        return f"{self.major}.{self.minor}.{self.patch}"


@dataclass(frozen=True)
class Project:
    root: Path
    name: str
    version: Version

    @classmethod
    def load(cls, root: Path) -> Self:
        package = tomllib.loads((root / "Cargo.toml").read_text())["package"]
        return cls(root, package["name"], Version.parse(package["version"]))

    def bump(self, part: str) -> None:
        version = self.version.bump(part)
        paths = [self.root / name for name in ("Cargo.toml", "Cargo.lock", "CHANGELOG.md")]
        original = {path: path.read_text() for path in paths}
        manifest, lockfile, changelog = (original[path] for path in paths)
        manifest = replace_version(manifest, r"\[package\]", self.version, version)
        lock_section = rf'\[\[package\]\]\s+name = "{re.escape(self.name)}"'
        lockfile = replace_version(lockfile, lock_section, self.version, version)
        assert changelog.startswith("# Changelog\n"), "Expected # Changelog heading"
        assert f"## {version}\n" not in changelog, f"Changelog already contains {version}"
        notes = compose_notes(version)
        changelog = f"# Changelog\n\n## {version}\n\n{notes}\n\n{changelog.removeprefix('# Changelog').lstrip()}"
        updated = dict(zip(paths, (manifest, lockfile, changelog)))
        assert all(path.read_text() == text for path, text in original.items()), "Project files changed while editing notes"
        try:
            for path, text in updated.items():
                path.write_text(text)
        except OSError:
            for path, text in original.items():
                path.write_text(text)
            raise
        print(f"{self.name}: {self.version} -> {version}")


def replace_version(text: str, section: str, old: Version, new: Version) -> str:
    pattern = rf'(?m)(^{section}[^\[]*?^version\s*=\s*"){re.escape(str(old))}("[^\n]*$)'
    updated, count = re.subn(pattern, lambda match: f"{match[1]}{new}{match[2]}", text)
    assert count == 1, "Expected exactly one matching package version"
    return updated


def compose_notes(version: Version) -> str:
    editor = shlex.split(os.environ.get("EDITOR", ""))
    assert editor, "Set EDITOR to an editor command (use its wait flag for GUI editors)"
    with tempfile.TemporaryDirectory(prefix="steam-workshop-notes-") as temporary:
        path = Path(temporary) / f"{version}.md"
        path.touch()
        print(f"Compose release notes for {version}; empty notes cancel the bump", flush=True)
        subprocess.run([*editor, str(path)], check=True)
        notes = path.read_text().strip()
        assert notes, "Empty release notes; version unchanged"
        assert not re.search(r"(?m)^#{1,2}\s", notes), "Use ### or deeper headings inside release notes"
        return notes


def main() -> None:
    assert len(sys.argv) == 2, "Usage: mise run bump major|minor|patch"
    Project.load(ROOT).bump(sys.argv[1])


if __name__ == "__main__":
    try:
        main()
    except (AssertionError, OSError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
