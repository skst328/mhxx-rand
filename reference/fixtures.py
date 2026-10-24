"""本家 (mhxx_rng.py) の関数で、mhxx-rand のテストの正解を作り、rand-core/tests/fixtures/reference.json に書き出す。

リポジトリ直下で uv run python reference/fixtures.py と流す。本家を写し直したときだけ作り直す。

お守りは [第 1 スキルの番号, ポイント, 第 2 スキルの番号 (なければ -1), ポイント (なければ 0), スロット] で、
鑑定画面に出る中身 (第 1 スキルと同じ第 2 スキルは、なしにする) で書く。
"""
import contextlib
import io
import json
import random
import re
from pathlib import Path

import fast
import mhxx_rng as m

OUT = Path(__file__).resolve().parents[1] / 'rand-core' / 'tests' / 'fixtures' / 'reference.json'
KINDS = ['weathered', 'worn', 'shining', 'mystery']
SETS = [m.set_blue, m.set_red, m.set_yellow, m.set_white]
MELDINGS = {'juju': (m.juju_colors, [0, 1]), 'halcyon': (m.halcyon_colors, [1, 2, 3])}
BILLION = 10**9

random.seed(20261024)
m.set_ja()
m.showallrank = False


def shown(c):
    """getcharm の結果を、鑑定画面に出る中身に"""
    has2 = c[2] != -1 and c[2] != c[0]
    return [c[0], c[1], c[2] if has2 else -1, c[3] if has2 else 0, c[4]]


def positions():
    """確かめる位置: 始まりの近く、乱数で選んだもの、10 億の手前"""
    return list(range(0, 40)) + sorted(random.randrange(BILLION) for _ in range(60)) + list(range(BILLION - 20, BILLION))


def charms():
    """位置ごとに、その位置から作るお守り (本家の jump してからの getcharm)"""
    out = []
    for kind, set_kind in zip(KINDS, SETS):
        set_kind()
        for p in positions():
            m.jump(p)
            out.append({'kind': kind, 'position': p, 'charm': shown(m.getcharm(0))})
    return out


def meldings():
    """錬金したフレームごとに、出るお守り (本家の meldingpush と同じ手順)"""
    out = []
    for melding, (colors, _) in MELDINGS.items():
        for rank in (1, 2, 3, 4):
            for frame in positions():
                m.jump(frame - 1)
                kinds = colors(m.r0, rank)
                m.roll()
                made = []
                for k in kinds:
                    SETS[k]()
                    made.append({'kind': KINDS[k], 'charm': shown(m.getcharm(0))})
                    for _ in range(4 if m.r2 % 100 < m.th else 6):
                        m.roll()
                out.append({'melding': melding, 'rank': rank, 'frame': frame, 'charms': made})
    return out


def aimpoints():
    """位置ごとに、そこに届く錬金のフレーム (本家の aimpoint_melding の ◯ の並び)。
    本家は位置 0 より前まで戻って数えるが、そこでは錬金できないので外す"""
    out = []
    for melding, (_, kinds) in MELDINGS.items():
        aim = m._aimpoint_juju if melding == 'juju' else m._aimpoint_halcyon
        for rank in (1, 2, 3, 4):
            for k in kinds:
                SETS[k]()
                for p in positions():
                    m.jump(p)
                    text = io.StringIO()
                    with contextlib.redirect_stdout(text):
                        aim(rank)
                    found = re.search(r'aim points exist(.*)', text.getvalue())
                    assert found, text.getvalue()
                    syms = [c for c in found.group(1) if c in '◯Ｘ']
                    starts = sorted(p - (len(syms) - 1 - i) for i, c in enumerate(syms) if c == '◯')
                    starts = [s for s in starts if s >= 0]
                    out.append({'melding': melding, 'rank': rank, 'kind': KINDS[k], 'position': p, 'starts': starts})
    return out


def searches():
    """お守りの検索 (本家の show_fast と同じものを、位置の一覧で)。見つかる数が少ないものは 10 億フレームで"""
    cases = [
        # (種類, 第 1 スキル, ポイント, 第 2 スキル, ポイント, スロット, 比べ方, 範囲)
        ('weathered', '連撃', 4, '痛撃', 5, 3, 'exact', BILLION),
        ('worn', '火攻', 3, '体力', 5, 1, 'exact', BILLION),
        ('weathered', '斬味', 7, '攻撃', 10, 3, 'at_least', BILLION),
        ('weathered', '斬味', 5, '', 0, 1, 'exact', 10**6),
        ('weathered', '斬味', 4, '攻撃', 5, 1, 'at_least', 10**7),
        ('worn', '火攻', 5, '', 0, 0, 'at_least', 10**5),
        ('shining', '攻撃', 4, '', 0, 2, 'exact', 10**6),
        ('mystery', '気絶', 3, '', 0, 0, 'at_least', 10**5),
    ]
    out = []
    for kind, s1, p1, s2, p2, slots, compare, end in cases:
        SETS[KINDS.index(kind)]()
        p = m.parameter(s1, p1, s2, p2, slots, 'マカ')
        found = fast.search_fast(0, end, 'search' if compare == 'exact' else 'search_greater', p)
        charm = [m.skill.index(s1), p1, m.skill.index(s2) if s2 else -1, p2 if s2 else 0, slots]
        out.append({'kind': kind, 'charm': charm, 'compare': compare, 'start': 0, 'end': end, 'positions': found})
    return out


def tables():
    """4 種類の表 (本家の set_blue などで設定される値そのもの)"""
    out = {}
    for kind, set_kind in zip(KINDS, SETS):
        set_kind()
        out[kind] = {
            'skill1': list(m.skill1), 'pts1': [list(r) for r in m.sp1],
            'skill2': list(m.skill2), 'pts2': [list(r) for r in m.sp2],
            'slots': [list(r) for r in m.slotvalue], 'no_skill2': m.th,
        }
    return out


def skill_names():
    """スキル番号ごとの名前 (本家の set_ja。2 文字に揃える全角スペースを取り、ＫＯ・ＳＰを半角にする)"""
    m.set_ja()
    return [s.rstrip('　').replace('ＫＯ', 'KO').replace('ＳＰ', 'SP') for s in m.skill]


def every_skill_searches():
    """どのスキルも最低 1 回は検索にかける: 4 種類の表の、第 1 スキルの全部の行と第 2 スキルの全部の行。
    第 1 スキルの行は、そのスキルの最小のポイント・第 2 スキルなし・スロット 0。第 2 スキルの行は、第 1 スキルを表の先頭
    (そのスキルと違うもの) の最小のポイントにし、第 2 スキルはポイント 1・スロット 0。同じもの (Exact) を探す。
    範囲は、見つかるまで 10 倍ずつ広げる (見つからないケースは検索にかけたことにならないので)"""
    out = []
    for kind, set_kind in zip(KINDS, SETS):
        set_kind()
        rows = [(s, m.sp1[i][0], -1, 0, 10**5) for i, s in enumerate(m.skill1)]
        if m.th < 100:  # なぞのお守りには第 2 スキルが付かない
            for s2 in m.skill2:
                i = next(i for i, s in enumerate(m.skill1) if s != s2)
                rows.append((m.skill1[i], m.sp1[i][0], s2, 1, 10**7))
        for s1, p1, s2, p2, end in rows:
            name = lambda i: m.skill[i] if i >= 0 else ''
            p = m.parameter(name(s1), p1, name(s2), p2, 0, 'マカ')
            found = fast.search_fast(0, end, 'search', p)
            while not found and end < BILLION:
                end *= 10
                found = fast.search_fast(0, end, 'search', p)
            assert found, (kind, s1, p1, s2, p2)
            out.append({'kind': kind, 'charm': [s1, p1, s2, p2, 0], 'compare': 'exact', 'start': 0, 'end': end,
                        'positions': found})
    return out


def combos():
    """調合の検索 (本家の search_combo)。本家の乱数で、位置 p から 5 個おきに生産数を作って累計の並びにし、
    0 から step 個の乱数を探した結果を読み取る。最後が上限 (99) に張り付いた並びも含める"""
    lut = m.combo_lookuptable()
    cases = [(12_345, 8, 10**6, False), (987_654, 20, 10**7, False), (5_000_000, 40, 10**7, False),
             (777, 30, 10**5, True), (3, 3, 10**4, False), (65_432_100, 25, 10**8, True)]
    out = []
    for p, crafts, step, capped in cases:
        m.jump(p)
        for _ in range(7):
            m.descend()  # jump の後は 7 つ進んでいるので、位置 p に戻す
        values = [0, 3, 6, 9]
        for _ in range(crafts):
            values.append(values[-1] + int(lut[(m.w & 0xFFFF) % 100]))
            for _ in range(5):
                m.ascend()
        if capped:
            values.append(99)
        text = io.StringIO()
        with contextlib.redirect_stdout(text):
            m.search_combo(0, step, ' '.join(f'{v:02d}' for v in values))
        found = [int(f) for f in re.findall(r'found! frame is… (\d+)', text.getvalue())]
        assert found, (p, crafts)
        out.append({'cumulative': values, 'step': step, 'frames': found})
    return out


def main():
    data = {'charms': charms(), 'meldings': meldings(), 'aimpoints': aimpoints(), 'searches': searches(),
            'every_skill_searches': every_skill_searches(), 'tables': tables(), 'skill_names': skill_names(),
            'combos': combos()}
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(data, ensure_ascii=False, separators=(',', ':')), encoding='utf-8')
    print(OUT, {k: len(v) for k, v in data.items()}, f'{OUT.stat().st_size // 1024} KB')


if __name__ == '__main__':
    main()
