"""調合の検索の Python の入口 (中身の正しさは rand-core のテストで見る)"""

import sys
from pathlib import Path

import pytest

import mhxx_rand as mr

# ずれのある並びを作るのに乱数が要るので、本家の写し (reference/) を使う
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "reference"))
import mhxx_rng as ref  # noqa: E402  # pyright: ignore[reportMissingImports]

LUT = ref.combo_lookuptable()


def cumulative(start, crafts, drift_at=None, drift=0):
    """位置 start から 5 個おきに生産数を作った累計の並び。drift_at の回の直後に、乱数が drift だけ余分に進む"""
    ref.jump(start)
    for _ in range(7):
        ref.descend()  # jump の後は 7 つ進んでいるので、位置 start に戻す
    values = [0, 3, 6, 9]
    for i in range(crafts):
        values.append(values[-1] + int(LUT[(ref.w & 0xFFFF) % 100]))
        for _ in range(5 + (drift if i == drift_at else 0)):
            ref.ascend()
    return values


def frame_of(start, values):
    """start から作った並びの報告値 (調合を始めた位置 + 補正)"""
    return start - 30 + 2 * (len(values) - 1)


def test_finds_the_frame_and_skips_diagnosis():
    values = cumulative(60_000, 30)
    r = mr.search_combo(mr.ComboQuery(values), 0, 100_000)
    assert frame_of(60_000, values) in r.frames
    assert r.diagnosis is None


def test_diagnoses_a_drift_when_not_found():
    values = cumulative(60_000, 30, drift_at=22, drift=1)
    q = mr.ComboQuery(values)
    r = mr.search_combo(q, 0, 100_000)
    assert r.frames == []
    d = r.diagnosis
    assert d is not None
    assert d.frame + d.total_drift == frame_of(60_000, values) + 1
    assert sum(x.steps for x in d.drifts) == 1
    assert mr.search_combo(q, 0, 100_000, diagnose=False).diagnosis is None


@pytest.mark.parametrize("nthreads", [2, 3])
def test_nthreads_give_same_result(nthreads):
    q = mr.ComboQuery([0, 3, 6, 9, 12, 14, 17, 21, 23])
    one = mr.search_combo(q, 0, 2_000_000)
    assert one.frames
    assert mr.search_combo(q, 0, 2_000_000, nthreads=nthreads).frames == one.frames
    # 並列でも、見つからなければ範囲全体で診断する
    drifted = mr.ComboQuery(cumulative(60_000, 30, drift_at=10, drift=-1))
    assert mr.search_combo(drifted, 0, 100_000, nthreads=nthreads).diagnosis is not None


@pytest.mark.parametrize(
    "values",
    [
        [0, 3, 5],  # 短すぎる
        [0, 3, 5, 7, None, 12],  # 分からない値
        [0, 3, 5, 7, 9, 10, 13],  # 差が 1
    ],
)
def test_unusable_sequences_are_value_errors(values):
    with pytest.raises(ValueError):
        mr.ComboQuery(values)


def test_pattern_len_and_reprs():
    q = mr.ComboQuery([0, 3, 5, 7, 10, 13, 99])
    assert q.pattern_len == 2
    values = cumulative(60_000, 30, drift_at=22, drift=1)
    r = mr.search_combo(mr.ComboQuery(values), 0, 100_000)
    assert repr(r).startswith("ComboFound(frames=[], diagnosis=Diagnosis(frame=")
    assert "Drift(craft=" in repr(r)
