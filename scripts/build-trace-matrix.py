#!/usr/bin/env python3
"""Regenerate docs/TRACE-MATRIX.md from requirements and Rust test markers.

Sources are ``docs/L1.md``, ``docs/L2.md``, ``docs/L3.md``, and
``/// Requirements: ...`` markers attached to Rust ``#[test]`` functions.

Usage:
    python scripts/build-trace-matrix.py
    python scripts/build-trace-matrix.py --check
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
L1_DOC = ROOT / "docs" / "L1.md"
L2_DOC = ROOT / "docs" / "L2.md"
L3_DOC = ROOT / "docs" / "L3.md"
TRACE_DOC = ROOT / "docs" / "TRACE-MATRIX.md"
RUST_SOURCE_ROOTS = (ROOT / "src", ROOT / "tests")

REQ_ID_PATTERN = re.compile(r"L(?P<level>[123])-(?P<cat>[A-Z]+)-(?P<num>\d+)")
L1_HEADER = re.compile(r"^###\s+(L1-[A-Z]+-\d+)\s*$", re.MULTILINE)
L2_HEADER = re.compile(r"^####\s+(L2-[A-Z]+-\d+)\s*$", re.MULTILINE)
L2_PARENT_LINE = re.compile(
    r"^\*\*Parent\*\*:\s+(L1-[A-Z]+-\d+)\s*$", re.MULTILINE
)
L3_ENTRY = re.compile(
    r"^\*\*(L3-[A-Z]+-\d+)\*\*\s+·\s+"
    r"Parent:\s+(L2-[A-Z]+-\d+)\s+·\s+"
    r"Applicability:\s+([^·\n]+?)\s+·\s+"
    r"Verification:\s+([^\n]+)$",
    re.MULTILINE,
)
CATEGORY_HEADER = re.compile(
    r"^##\s+L1-([A-Z]+):\s+(.+?)\s*$", re.MULTILINE
)
VERIFICATION_LINE = re.compile(
    r"^\*\*Verification Method\*\*:\s+([^\n]+)$", re.MULTILINE
)
EVIDENCE_LINE = re.compile(r"^\*\*Evidence\*\*:\s+([^\n]+)$", re.MULTILINE)
BACKTICKED = re.compile(r"`([^`]+)`")
METHOD_LETTER = re.compile(r"\b([TIADE])\b")
FN_DECL = re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*[(<]")


def _blocks(doc: str, header: re.Pattern[str]) -> list[tuple[str, str]]:
    """Return ``(requirement id, body)`` pairs for a Markdown header."""
    matches = list(header.finditer(doc))
    return [
        (
            match.group(1),
            doc[match.end() : matches[index + 1].start()]
            if index + 1 < len(matches)
            else doc[match.end() :],
        )
        for index, match in enumerate(matches)
    ]


def _methods(text: str) -> set[str]:
    return set(METHOD_LETTER.findall(text))


def _evidence(text: str) -> list[str]:
    found = BACKTICKED.findall(text)
    if found:
        return found
    stripped = text.strip()
    return [stripped] if stripped else []


def _verification_metadata(
    blocks: list[tuple[str, str]],
) -> tuple[dict[str, set[str]], dict[str, list[str]]]:
    methods: dict[str, set[str]] = {}
    evidence: dict[str, list[str]] = {}
    for req_id, body in blocks:
        verification = VERIFICATION_LINE.search(body)
        if verification:
            methods[req_id] = _methods(verification.group(1))
        artifact = EVIDENCE_LINE.search(body)
        if artifact:
            evidence[req_id] = _evidence(artifact.group(1))
    return methods, evidence


def parse_l1(
    doc: str,
) -> tuple[list[str], dict[str, set[str]], dict[str, list[str]]]:
    blocks = _blocks(doc, L1_HEADER)
    methods, evidence = _verification_metadata(blocks)
    return [req_id for req_id, _ in blocks], methods, evidence


def parse_l2(
    doc: str,
) -> tuple[dict[str, str], dict[str, set[str]], dict[str, list[str]]]:
    blocks = _blocks(doc, L2_HEADER)
    parents: dict[str, str] = {}
    for req_id, body in blocks:
        parent = L2_PARENT_LINE.search(body)
        if parent:
            parents[req_id] = parent.group(1)
    methods, evidence = _verification_metadata(blocks)
    return parents, methods, evidence


def parse_l3(
    doc: str,
) -> tuple[
    dict[str, str],
    dict[str, str],
    dict[str, set[str]],
    dict[str, list[str]],
]:
    parents: dict[str, str] = {}
    applicability: dict[str, str] = {}
    methods: dict[str, set[str]] = {}
    evidence: dict[str, list[str]] = {}
    matches = list(L3_ENTRY.finditer(doc))
    for index, match in enumerate(matches):
        req_id, parent, applies, verification = match.groups()
        body_end = matches[index + 1].start() if index + 1 < len(matches) else len(doc)
        body = doc[match.end() : body_end]
        parents[req_id] = parent
        applicability[req_id] = applies.strip()
        methods[req_id] = _methods(verification)
        artifact = EVIDENCE_LINE.search(body)
        if artifact:
            evidence[req_id] = _evidence(artifact.group(1))
    return parents, applicability, methods, evidence


def collect_rust_markers() -> dict[str, list[str]]:
    """Collect requirement IDs attached to Rust test functions."""
    marker_map: dict[str, list[str]] = defaultdict(list)
    for source_root in RUST_SOURCE_ROOTS:
        if not source_root.is_dir():
            continue
        for rust_file in sorted(source_root.rglob("*.rs")):
            try:
                source = rust_file.read_text(encoding="utf-8")
            except OSError:
                continue
            relative = rust_file.relative_to(ROOT).as_posix()
            pending_ids: list[str] = []
            saw_test_attribute = False
            attribute_depth = 0
            for line in source.splitlines():
                stripped = line.strip()
                if attribute_depth > 0:
                    attribute_depth += stripped.count("[") - stripped.count("]")
                    continue
                if stripped.startswith("///") and "Requirements:" in stripped:
                    _, _, marker_text = stripped.partition("Requirements:")
                    pending_ids.extend(
                        match.group(0)
                        for match in REQ_ID_PATTERN.finditer(marker_text)
                    )
                    continue
                if stripped.startswith("#["):
                    if (
                        stripped.startswith("#[test")
                        or "::test]" in stripped
                        or stripped.startswith("#[rstest")
                    ):
                        saw_test_attribute = True
                    attribute_depth = stripped.count("[") - stripped.count("]")
                    continue
                if stripped.startswith("//") or not stripped:
                    continue
                function = FN_DECL.match(line)
                if function and saw_test_attribute and pending_ids:
                    artifact = f"{relative}::{function.group(1)}"
                    for req_id in pending_ids:
                        marker_map[req_id].append(artifact)
                    pending_ids = []
                    saw_test_attribute = False
                    continue
                pending_ids = []
                saw_test_attribute = False
    return {
        req_id: sorted(set(artifacts))
        for req_id, artifacts in marker_map.items()
    }


def _sort_key(req_id: str) -> tuple[str, int]:
    match = REQ_ID_PATTERN.fullmatch(req_id)
    if not match:
        return req_id, 0
    return match.group("cat"), int(match.group("num"))


def _non_test_status(
    methods: set[str] | None, evidence: list[str] | None
) -> str | None:
    if not methods or "T" in methods or not evidence:
        return None
    return f"Implemented ({'+'.join(sorted(methods))})"


def compute_status(
    *,
    direct_artifacts: list[str],
    child_statuses: list[str],
    methods: set[str] | None,
    evidence: list[str] | None,
) -> str:
    """Roll direct evidence and child status into one requirement status."""
    if not child_statuses:
        if direct_artifacts:
            return "Implemented"
        return _non_test_status(methods, evidence) or "Draft"
    implemented = sum(status.startswith("Implemented") for status in child_statuses)
    draft = sum(status == "Draft" for status in child_statuses)
    if implemented == len(child_statuses):
        return "Implemented"
    if draft == len(child_statuses) and not direct_artifacts:
        return _non_test_status(methods, evidence) or "Draft"
    return "Partially Implemented"


def _is_verified(
    req_id: str,
    markers: dict[str, list[str]],
    methods: dict[str, set[str]],
    evidence: dict[str, list[str]],
) -> bool:
    return bool(markers.get(req_id)) or bool(
        methods.get(req_id)
        and "T" not in methods[req_id]
        and evidence.get(req_id)
    )


def build_matrix() -> str:
    l1_doc = L1_DOC.read_text(encoding="utf-8")
    l2_doc = L2_DOC.read_text(encoding="utf-8")
    l3_doc = L3_DOC.read_text(encoding="utf-8")
    l1_ids, l1_methods, l1_evidence = parse_l1(l1_doc)
    l2_parent, l2_methods, l2_evidence = parse_l2(l2_doc)
    l3_parent, l3_applicability, l3_methods, l3_evidence = parse_l3(l3_doc)
    markers = collect_rust_markers()

    l1_to_l2: dict[str, list[str]] = defaultdict(list)
    for l2_id, l1_id in l2_parent.items():
        l1_to_l2[l1_id].append(l2_id)
    for children in l1_to_l2.values():
        children.sort(key=_sort_key)
    l2_to_l3: dict[str, list[str]] = defaultdict(list)
    for l3_id, l2_id in l3_parent.items():
        l2_to_l3[l2_id].append(l3_id)
    for children in l2_to_l3.values():
        children.sort(key=_sort_key)

    def l3_status(req_id: str) -> str:
        return compute_status(
            direct_artifacts=markers.get(req_id, []),
            child_statuses=[],
            methods=l3_methods.get(req_id),
            evidence=l3_evidence.get(req_id),
        )

    def l2_status(req_id: str) -> str:
        return compute_status(
            direct_artifacts=markers.get(req_id, []),
            child_statuses=[l3_status(child) for child in l2_to_l3.get(req_id, [])],
            methods=l2_methods.get(req_id),
            evidence=l2_evidence.get(req_id),
        )

    categories = CATEGORY_HEADER.findall(l1_doc)
    lines = [
        "# Aero1394 — Requirements trace matrix",
        "",
        "<!-- AUTO-GENERATED by scripts/build-trace-matrix.py. Do not edit by hand. -->",
        "",
        "## Purpose",
        "",
        "Forward trace from `L1.md` through `L2.md` and `L3.md` to Rust test",
        "artifacts. Run `python scripts/build-trace-matrix.py` to regenerate this",
        "file or add `--check` to detect drift without writing.",
        "",
        "## Status rollup",
        "",
        "- **Draft** — test verification is required but no test marker exists, or",
        "  non-test verification has no named evidence artifact.",
        "- **Implemented** — a leaf has a test marker, or every child is implemented.",
        "- **Implemented (I/A/D/E)** — a non-test leaf names the artifact carrying",
        "  its inspection, analysis, demonstration, or evidence comparison.",
        "- **Partially Implemented** — at least one child is implemented and at",
        "  least one remains draft.",
        "",
        "---",
        "",
    ]

    for category, title in categories:
        category_l1 = [req for req in l1_ids if req.startswith(f"L1-{category}-")]
        if not category_l1:
            continue
        lines.extend(
            [
                f"### L1-{category}: {title}",
                "",
                "**L1 -> L2**",
                "",
                "| L1 ID | L2 children | Direct artifacts | Status |",
                "| --- | --- | --- | --- |",
            ]
        )
        for l1_id in category_l1:
            children = l1_to_l2.get(l1_id, [])
            child_text = ", ".join(children) if children else "_(none)_"
            direct = markers.get(l1_id, []) + l1_evidence.get(l1_id, [])
            artifact_text = "<br>".join(
                f"`{item}`" for item in sorted(set(direct))
            ) or "_(none)_"
            status = compute_status(
                direct_artifacts=markers.get(l1_id, []),
                child_statuses=[l2_status(child) for child in children],
                methods=l1_methods.get(l1_id),
                evidence=l1_evidence.get(l1_id),
            )
            lines.append(f"| {l1_id} | {child_text} | {artifact_text} | {status} |")

        lines.extend(
            [
                "",
                "**L2 -> L3 -> Verification artifacts**",
                "",
                "| L2 ID | L3 children | Verification artifacts | Status |",
                "| --- | --- | --- | --- |",
            ]
        )
        category_l2 = sorted(
            [req for req, parent in l2_parent.items() if parent in category_l1],
            key=_sort_key,
        )
        for l2_id in category_l2:
            children = l2_to_l3.get(l2_id, [])
            child_text = ", ".join(children) if children else "_(none)_"
            artifacts = list(markers.get(l2_id, [])) + list(l2_evidence.get(l2_id, []))
            for child in children:
                artifacts.extend(markers.get(child, []))
                artifacts.extend(l3_evidence.get(child, []))
            artifact_text = "<br>".join(
                f"`{item}`" for item in sorted(set(artifacts))
            ) or "_(TBD)_"
            lines.append(
                f"| {l2_id} | {child_text} | {artifact_text} | {l2_status(l2_id)} |"
            )
        lines.append("")

    lines.extend(
        [
            "---",
            "",
            "## Coverage summary",
            "",
            "- **Tested** means at least one `/// Requirements:` marker names the",
            "  requirement.",
            "- **Verified** means tested, or a non-test requirement names its",
            "  evidence artifact.",
            "",
            "| Category | L1 | L2 | L3 | L2 tested | L3 tested | L2 verified | L3 verified |",
            "| --- | --- | --- | --- | --- | --- | --- | --- |",
        ]
    )
    totals = [0] * 7
    for category, _ in categories:
        l1s = [req for req in l1_ids if req.startswith(f"L1-{category}-")]
        l2s = [req for req in l2_parent if req.startswith(f"L2-{category}-")]
        l3s = [req for req in l3_parent if req.startswith(f"L3-{category}-")]
        row = [
            len(l1s),
            len(l2s),
            len(l3s),
            sum(bool(markers.get(req)) for req in l2s),
            sum(bool(markers.get(req)) for req in l3s),
            sum(_is_verified(req, markers, l2_methods, l2_evidence) for req in l2s),
            sum(_is_verified(req, markers, l3_methods, l3_evidence) for req in l3s),
        ]
        totals = [total + value for total, value in zip(totals, row)]
        lines.append(f"| {category} | " + " | ".join(str(value) for value in row) + " |")
    lines.append("| **Total** | " + " | ".join(f"**{value}**" for value in totals) + " |")
    lines.append("")

    l1_leaves = [req for req in l1_ids if not l1_to_l2.get(req)]
    countable = len(l2_parent) + len(l3_parent) + len(l1_leaves)
    tested = (
        sum(bool(markers.get(req)) for req in l2_parent)
        + sum(bool(markers.get(req)) for req in l3_parent)
        + sum(bool(markers.get(req)) for req in l1_leaves)
    )
    verified = (
        sum(_is_verified(req, markers, l2_methods, l2_evidence) for req in l2_parent)
        + sum(_is_verified(req, markers, l3_methods, l3_evidence) for req in l3_parent)
        + sum(_is_verified(req, markers, l1_methods, l1_evidence) for req in l1_leaves)
    )
    if countable:
        lines.extend(
            [
                f"**Tested by at least one marker**: {tested} of {countable} "
                f"({tested * 100 / countable:.1f}%).",
                "",
                f"**Verified by a test or named non-test evidence**: {verified} of "
                f"{countable} ({verified * 100 / countable:.1f}%).",
                "",
            ]
        )

    applicability_counts: dict[str, int] = defaultdict(int)
    for value in l3_applicability.values():
        applicability_counts[value] += 1
    lines.extend(
        [
            "### Applicability summary",
            "",
            "| Applicability | L3 requirements |",
            "| --- | --- |",
        ]
    )
    known_applicability = ("Active", "Evidence-limited", "Deferred")
    for applicability in known_applicability:
        lines.append(f"| {applicability} | {applicability_counts.get(applicability, 0)} |")
    for applicability in sorted(set(applicability_counts) - set(known_applicability)):
        lines.append(f"| {applicability} | {applicability_counts[applicability]} |")
    lines.append("")

    orphan_l2 = sorted(
        (req for req, parent in l2_parent.items() if parent not in l1_ids),
        key=_sort_key,
    )
    orphan_l3 = sorted(
        (req for req, parent in l3_parent.items() if parent not in l2_parent),
        key=_sort_key,
    )
    lines.extend(
        [
            "### Orphan check",
            "",
            f"- Orphan L2s (parent L1 not found): **{len(orphan_l2)}**",
            f"- Orphan L3s (parent L2 not found): **{len(orphan_l3)}**",
        ]
    )
    for req_id in orphan_l2:
        lines.append(f"- `{req_id}` -> missing `{l2_parent[req_id]}`")
    for req_id in orphan_l3:
        lines.append(f"- `{req_id}` -> missing `{l3_parent[req_id]}`")
    lines.append("")

    known = set(l1_ids) | set(l2_parent) | set(l3_parent)
    unknown_markers = sorted(set(markers) - known, key=_sort_key)
    lines.extend(
        [
            "### Marker reference check",
            "",
            f"- Markers referencing unknown requirement IDs: **{len(unknown_markers)}**",
        ]
    )
    for req_id in unknown_markers:
        lines.append(f"- `{req_id}` — referenced by {len(markers[req_id])} test(s)")

    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="Do not write; fail if docs/TRACE-MATRIX.md would change.",
    )
    args = parser.parse_args(argv)
    generated = build_matrix()
    if args.check:
        try:
            current = TRACE_DOC.read_bytes().decode("utf-8")
        except OSError:
            current = ""
        if current != generated:
            print(
                "docs/TRACE-MATRIX.md is out of date. Run "
                "`python scripts/build-trace-matrix.py`.",
                file=sys.stderr,
            )
            return 1
        return 0
    TRACE_DOC.write_bytes(generated.encode("utf-8"))
    print("Wrote docs/TRACE-MATRIX.md")
    return 0


if __name__ == "__main__":
    sys.exit(main())
