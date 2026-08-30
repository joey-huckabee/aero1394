#!/usr/bin/env python3
"""Build and smoke-test deterministic Aero1394 release archives."""

from __future__ import annotations

import argparse
from datetime import date
import gzip
import hashlib
import io
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile
import tomllib
import zipfile


ROOT = Path(__file__).resolve().parent.parent
FIXED_ZIP_TIME = (1980, 1, 1, 0, 0, 0)
SAFE_LABEL = re.compile(r"^[A-Za-z0-9._-]+$")


def package_version() -> str:
    with (ROOT / "Cargo.toml").open("rb") as manifest:
        data = tomllib.load(manifest)
    return str(data["package"]["version"])


def verify_release_metadata(version: str) -> None:
    changelog = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    heading = re.search(
        rf"^## \[{re.escape(version)}\] - (?P<date>\d{{4}}-\d{{2}}-\d{{2}})$",
        changelog,
        re.MULTILINE,
    )
    if heading is None:
        raise RuntimeError(
            f"CHANGELOG.md must contain a dated [{version}] heading before tagging"
        )
    try:
        date.fromisoformat(heading.group("date"))
    except ValueError as error:
        raise RuntimeError(
            f"CHANGELOG.md has an invalid release date for [{version}]"
        ) from error

    release_notes = ROOT / "docs" / f"RELEASE-NOTES-v{version}.md"
    notes = release_notes.read_text(encoding="utf-8")
    expected_title = f"# Aero1394 v{version} release notes"
    if notes.splitlines()[:1] != [expected_title]:
        raise RuntimeError(
            f"{release_notes.name} must start with {expected_title!r} before tagging"
        )
    if "Status: Unreleased" in notes:
        raise RuntimeError(
            f"{release_notes.name} must not be marked unreleased before tagging"
        )


def verify_tag(version: str) -> None:
    git_ref = os.environ.get("GITHUB_REF", "")
    if git_ref.startswith("refs/tags/"):
        actual = git_ref.removeprefix("refs/tags/")
        expected = f"v{version}"
        if actual != expected:
            raise RuntimeError(
                f"release tag {actual!r} does not match Cargo version tag {expected!r}"
            )
        verify_release_metadata(version)


def smoke_test(binary: Path, version: str) -> None:
    version_result = subprocess.run(
        [str(binary), "--version"],
        check=False,
        capture_output=True,
        text=True,
    )
    expected_version = f"aero1394 {version}\n"
    if (
        version_result.returncode != 0
        or version_result.stdout != expected_version
        or version_result.stderr
    ):
        raise RuntimeError(
            f"version smoke test failed for {binary}: "
            f"exit={version_result.returncode}, stdout={version_result.stdout!r}, "
            f"stderr={version_result.stderr!r}"
        )

    help_checks = (
        (["--help"], "aero1394 <COMMAND>"),
        (["hexdump", "--help"], "aero1394 hexdump <FILE> [OPTIONS]"),
        (["records", "--help"], "aero1394 records <FILE>"),
        (["as5643", "--help"], "aero1394 as5643 <FILE>"),
    )
    for arguments, expected_usage in help_checks:
        help_result = subprocess.run(
            [str(binary), *arguments],
            check=False,
            capture_output=True,
            text=True,
        )
        if (
            help_result.returncode != 0
            or expected_usage not in help_result.stdout
            or help_result.stderr
        ):
            command = " ".join(arguments)
            raise RuntimeError(
                f"{command} smoke test failed for {binary}: "
                f"exit={help_result.returncode}, stdout={help_result.stdout!r}, "
                f"stderr={help_result.stderr!r}"
            )


def package_members(binary: Path, version: str) -> list[tuple[str, bytes, int]]:
    release_notes = ROOT / "docs" / f"RELEASE-NOTES-v{version}.md"
    sources = [
        (binary.name, binary, 0o755),
        ("LICENSE", ROOT / "LICENSE", 0o644),
        ("README.md", ROOT / "README.md", 0o644),
        ("RELEASE-NOTES.md", release_notes, 0o644),
    ]
    missing = [str(source) for _, source, _ in sources if not source.is_file()]
    if missing:
        raise FileNotFoundError(f"release inputs are missing: {', '.join(missing)}")
    return [(name, source.read_bytes(), mode) for name, source, mode in sources]


def write_zip(
    archive: Path,
    top_level: str,
    members: list[tuple[str, bytes, int]],
) -> None:
    with zipfile.ZipFile(
        archive,
        mode="w",
        compression=zipfile.ZIP_DEFLATED,
        compresslevel=9,
    ) as package:
        for name, data, mode in members:
            entry = zipfile.ZipInfo(f"{top_level}/{name}", FIXED_ZIP_TIME)
            entry.create_system = 3
            entry.compress_type = zipfile.ZIP_DEFLATED
            entry.external_attr = mode << 16
            package.writestr(entry, data, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)


def write_tar_gz(
    archive: Path,
    top_level: str,
    members: list[tuple[str, bytes, int]],
) -> None:
    with archive.open("wb") as raw_archive:
        with gzip.GzipFile(
            filename="",
            mode="wb",
            compresslevel=9,
            fileobj=raw_archive,
            mtime=0,
        ) as compressed:
            with tarfile.open(
                fileobj=compressed,
                mode="w",
                format=tarfile.USTAR_FORMAT,
            ) as package:
                for name, data, mode in members:
                    entry = tarfile.TarInfo(f"{top_level}/{name}")
                    entry.size = len(data)
                    entry.mode = mode
                    entry.mtime = 0
                    entry.uid = 0
                    entry.gid = 0
                    entry.uname = ""
                    entry.gname = ""
                    package.addfile(entry, io.BytesIO(data))


def smoke_test_archive(
    archive: Path,
    archive_format: str,
    top_level: str,
    binary_name: str,
    version: str,
    member_names: list[str],
) -> None:
    with tempfile.TemporaryDirectory(prefix="aero1394-package-") as directory:
        extraction_root = Path(directory)
        expected_paths = [f"{top_level}/{name}" for name in member_names]
        if archive_format == "zip":
            with zipfile.ZipFile(archive) as package:
                if package.namelist() != expected_paths:
                    raise RuntimeError(
                        f"ZIP member mismatch: expected {expected_paths!r}, "
                        f"found {package.namelist()!r}"
                    )
                package.extractall(extraction_root)
        else:
            with tarfile.open(archive, mode="r:gz") as package:
                if package.getnames() != expected_paths:
                    raise RuntimeError(
                        f"tar member mismatch: expected {expected_paths!r}, "
                        f"found {package.getnames()!r}"
                    )
                package.extractall(extraction_root, filter="data")
        smoke_test(extraction_root / top_level / binary_name, version)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--platform", required=True, help="artifact platform label")
    parser.add_argument(
        "--archive-format",
        required=True,
        choices=("zip", "tar.gz"),
        help="release archive format",
    )
    parser.add_argument("--binary", required=True, type=Path, help="release binary path")
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=ROOT / "dist",
        help="artifact output directory",
    )
    return parser.parse_args()


def display_path(path: Path) -> Path:
    try:
        return path.relative_to(ROOT)
    except ValueError:
        return path


def main() -> int:
    args = parse_args()
    if not SAFE_LABEL.fullmatch(args.platform):
        raise ValueError(f"unsafe platform label: {args.platform!r}")

    version = package_version()
    verify_tag(version)
    binary = args.binary if args.binary.is_absolute() else ROOT / args.binary
    binary = binary.resolve()
    if not binary.is_file():
        raise FileNotFoundError(f"release binary does not exist: {binary}")
    smoke_test(binary, version)

    output_dir = args.output_dir if args.output_dir.is_absolute() else ROOT / args.output_dir
    output_dir.mkdir(parents=True, exist_ok=True)
    base_name = f"aero1394-v{version}-{args.platform}"
    suffix = ".zip" if args.archive_format == "zip" else ".tar.gz"
    archive = output_dir / f"{base_name}{suffix}"
    members = package_members(binary, version)

    if args.archive_format == "zip":
        write_zip(archive, base_name, members)
    else:
        write_tar_gz(archive, base_name, members)

    smoke_test_archive(
        archive,
        args.archive_format,
        base_name,
        binary.name,
        version,
        [name for name, _, _ in members],
    )
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    checksum = archive.with_name(f"{archive.name}.sha256")
    checksum.write_bytes(f"{digest}  {archive.name}\n".encode("ascii"))
    print(display_path(archive))
    print(display_path(checksum))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
