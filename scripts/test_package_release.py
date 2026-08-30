"""Tests for tag-only release metadata gates."""

from __future__ import annotations

import importlib.util
import os
from pathlib import Path
import sys
import unittest
from unittest import mock


SCRIPT = Path(__file__).with_name("package-release.py")
SPEC = importlib.util.spec_from_file_location("package_release", SCRIPT)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"could not load {SCRIPT}")
package_release = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = package_release
SPEC.loader.exec_module(package_release)


class VerifyTagTests(unittest.TestCase):
    @staticmethod
    def metadata(*, finalized: bool) -> tuple[str, str]:
        if finalized:
            changelog_heading = "## [0.2.0] - 2026-08-30"
            notes_title = "# Aero1394 v0.2.0 release notes"
            notes_status = ""
        else:
            changelog_heading = "## [Unreleased]"
            notes_title = "# Aero1394 v0.2.0 release notes (draft)"
            notes_status = "\n- Status: Unreleased"
        return (
            f"# Changelog\n\n{changelog_heading}\n",
            f"{notes_title}\n{notes_status}\n",
        )

    def verify(
        self,
        git_ref: str,
        *,
        finalized: bool | None = None,
        changelog_override: str | None = None,
        notes_override: str | None = None,
    ) -> None:
        changelog, notes = self.metadata(finalized=bool(finalized))
        changelog = changelog_override or changelog
        notes = notes_override or notes

        def read_text(path: Path, **_: object) -> str:
            if path.name == "CHANGELOG.md":
                return changelog
            if path.name == "RELEASE-NOTES-v0.2.0.md":
                return notes
            raise AssertionError(f"unexpected metadata path: {path}")

        with (
            mock.patch.dict(os.environ, {"GITHUB_REF": git_ref}, clear=False),
            mock.patch.object(package_release.Path, "read_text", read_text),
        ):
            package_release.verify_tag("0.2.0")

    def test_non_tag_build_does_not_require_finalized_metadata(self) -> None:
        self.verify("refs/heads/main")

    def test_matching_tag_requires_finalized_metadata(self) -> None:
        with self.assertRaisesRegex(RuntimeError, "dated.*heading"):
            self.verify("refs/tags/v0.2.0", finalized=False)

    def test_matching_tag_accepts_finalized_metadata(self) -> None:
        self.verify("refs/tags/v0.2.0", finalized=True)

    def test_matching_tag_rejects_an_invalid_release_date(self) -> None:
        with self.assertRaisesRegex(RuntimeError, "invalid release date"):
            self.verify(
                "refs/tags/v0.2.0",
                changelog_override="# Changelog\n\n## [0.2.0] - 2026-02-30\n",
            )

    def test_matching_tag_rejects_draft_release_notes(self) -> None:
        with self.assertRaisesRegex(RuntimeError, "must start"):
            self.verify(
                "refs/tags/v0.2.0",
                finalized=True,
                notes_override="# Aero1394 v0.2.0 release notes (draft)\n",
            )

    def test_mismatched_tag_is_rejected(self) -> None:
        with self.assertRaisesRegex(RuntimeError, "does not match"):
            self.verify("refs/tags/v0.2.1")


if __name__ == "__main__":
    unittest.main()
