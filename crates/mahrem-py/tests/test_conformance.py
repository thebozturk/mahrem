"""Ortak conformance seti: Rust ve Python aynı dosyayı okur."""

import json
from pathlib import Path

import pytest

import mahrem

CONFORMANCE = Path(__file__).resolve().parents[3] / "conformance"


def _load(name: str) -> list[dict]:
    text = (CONFORMANCE / name).read_text("utf-8")
    return [json.loads(line) for line in text.splitlines() if line.strip()]


CASES = _load("validators.jsonl")
ANALYZE_CASES = _load("analyze.jsonl")


@pytest.mark.parametrize("case", CASES, ids=lambda c: f"{c['func']}:{c['input']}")
def test_conformance(case: dict) -> None:
    func = getattr(mahrem, case["func"])
    assert func(case["input"]) == case["expected"]


@pytest.mark.parametrize("case", ANALYZE_CASES, ids=lambda c: c["text"][:40])
def test_analyze_conformance(case: dict) -> None:
    got = [
        {"label": e.label, "start": e.start, "end": e.end, "text": e.text, "source": e.source}
        for e in mahrem.analyze(case["text"])
    ]
    assert got == case["expected"]
    for e in mahrem.analyze(case["text"]):
        assert case["text"][e.start : e.end] == e.text


def test_analyze_threshold() -> None:
    text = "Sipariş 1234567890 kargoya verildi."
    assert mahrem.analyze(text) == []
    low = mahrem.analyze(text, min_score=0.3)
    assert [e.label for e in low] == ["VKN"]
    assert low[0].score < mahrem.DEFAULT_MIN_SCORE
    assert "VKN" in repr(low[0])


def test_version() -> None:
    assert mahrem.__version__
