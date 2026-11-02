#!/usr/bin/env python3
import json
import os
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Self

ROOT = Path(__file__).resolve().parents[1]


def gh(*args: str) -> str:
    return subprocess.run(["gh", *args], cwd=ROOT, check=True, capture_output=True, text=True).stdout


def git(*args: str) -> str:
    return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True, text=True).stdout


def version_tuple(version: str) -> tuple[int, ...]:
    assert re.fullmatch(r"(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)", version), f"Unsupported version: {version}"
    return tuple(map(int, version.split(".")))


@dataclass(frozen=True)
class Release:
    version: str
    repository: str
    commit: str

    @classmethod
    def load(cls) -> Self:
        package = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]
        version_tuple(package["version"])
        return cls(package["version"], os.environ["GITHUB_REPOSITORY"], os.environ["GITHUB_SHA"])

    @property
    def tag(self) -> str:
        return f"v{self.version}"

    @property
    def notes_path(self) -> Path:
        return Path(os.environ["RUNNER_TEMP"]) / "release-notes.md"

    def notes(self) -> str:
        changelog = (ROOT / "CHANGELOG.md").read_text()
        sections = re.findall(rf"(?ms)^## {re.escape(self.version)}\n(.*?)(?=^## |\Z)", changelog)
        assert len(sections) == 1 and sections[0].strip(), f"Missing or duplicate changelog entry: {self.version}"
        return sections[0].strip() + "\n"

    def status(self) -> str:
        pages = json.loads(gh("api", f"repos/{self.repository}/releases", "--paginate", "--slurp"))
        releases = [release for page in pages for release in page]
        existing = next((release for release in releases if release["tag_name"] == self.tag), None)
        if existing and not existing["draft"]:
            return "published"
        if existing:
            assert existing["target_commitish"] == self.commit, "Draft belongs to a different commit"
        for release in releases:
            if not release["draft"] and not release["prerelease"]:
                assert version_tuple(self.version) > version_tuple(release["tag_name"].removeprefix("v")), (
                    "Version must increase"
                )
        return "draft" if existing else "new"

    def prepare(self) -> None:
        pending = self.status() != "published"
        if pending:
            self.notes_path.write_text(self.notes())
        with Path(os.environ["GITHUB_OUTPUT"]).open("a") as output:
            output.write(f"pending={str(pending).lower()}\n")
        print(f"{self.tag}: {'ready to build' if pending else 'already published'}")

    def publish(self) -> None:
        status = self.status()
        if status == "published":
            return
        self.notes_path.write_text(self.notes())
        archive = ROOT / "target" / "dist" / f"steam-workshop-{self.tag}-aarch64-apple-darwin.tar.gz"
        checksum = archive.with_name(f"{archive.name}.sha256")
        assert archive.is_file() and checksum.is_file(), "Missing release assets"
        tags = git("tag", "--list", self.tag)
        if tags.strip():
            commit = git("rev-list", "-n", "1", self.tag).strip()
            assert commit == self.commit, "Tag belongs to a different commit"
        if status == "new":
            gh(
                "release",
                "create",
                self.tag,
                "--repo",
                self.repository,
                "--target",
                self.commit,
                "--draft",
                "--title",
                self.tag,
                "--notes-file",
                str(self.notes_path),
            )
        gh("release", "upload", self.tag, str(archive), str(checksum), "--repo", self.repository, "--clobber")
        gh(
            "release",
            "edit",
            self.tag,
            "--repo",
            self.repository,
            "--notes-file",
            str(self.notes_path),
            "--draft=false",
            "--latest",
        )
        print(f"Published {self.tag}")


def main() -> None:
    assert len(sys.argv) == 2 and sys.argv[1] in ("prepare", "publish"), "Usage: publish.py prepare|publish"
    release = Release.load()
    {"prepare": release.prepare, "publish": release.publish}[sys.argv[1]]()


if __name__ == "__main__":
    main()
