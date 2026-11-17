"""Ortak conformance seti: Rust ve Python aynı dosyayı okur."""

import json
from pathlib import Path

import pytest

import mahrem

CASES_FILE = Path(__file__).resolve().parents[3] / "conformance" / "validators.jsonl"
CASES = [json.loads(line) for line in CASES_FILE.read_text("utf-8").splitlines() if line.strip()]


@pytest.mark.parametrize("case", CASES, ids=lambda c: f"{c['func']}:{c['input']}")
def test_conformance(case: dict) -> None:
    func = getattr(mahrem, case["func"])
    assert func(case["input"]) == case["expected"]


def test_version() -> None:
    assert mahrem.__version__
