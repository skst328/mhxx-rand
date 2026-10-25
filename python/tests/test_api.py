"""Python の入口の振る舞い (中身の正しさは rand-core のテストで見る)"""

import datetime

import pytest

import mhxx_rand as mr

WANT = mr.Charm(skill1=mr.Skill("連撃"), pts1=4, skill2=mr.Skill("痛撃"), pts2=5, slots=3)


def test_search_finds_the_readme_example():
    found = mr.search(mr.Query(mr.Melding.Juju, WANT, kind=mr.Kind.Weathered), 0, 10_000)
    assert [f.position for f in found] == [4723]
    f = found[0]
    assert (f.kind, f.index, f.charm) == (mr.Kind.Weathered, 0, WANT)
    assert [(a.start, a.index, str(a.game_time())) for a in f.aimpoints()] == [(4723, 1, "0d 0h 2m 37s 13f")]


@pytest.mark.parametrize("nthreads", [2, 3, 7])
def test_nthreads_give_same_result(nthreads):
    q = mr.Query(mr.Melding.Juju, mr.Charm(mr.Skill("聴覚"), 3))
    key = lambda fs: [(f.position, f.kind, f.index) for f in fs]
    assert key(mr.search(q, 5, 1_000_003, mr.Compare.AtLeast, nthreads=nthreads)) == key(
        mr.search(q, 5, 1_000_003, mr.Compare.AtLeast)
    )
    qs = [q, mr.Query(mr.Melding.Halcyon, mr.Charm(mr.Skill("体力"), 3))]
    assert key(mr.search_many(qs, 0, 1_000_000, nthreads=nthreads)) == key(mr.search_many(qs, 0, 1_000_000))


def test_skill_names_and_ids():
    assert (mr.Skill("連撃").id, mr.Skill("連撃").name) == (71, "連撃")
    assert mr.Skill.from_id(86) == mr.Skill("KO")
    assert mr.Skill("毒") == mr.Skill.from_id(0)
    assert len(mr.Skill.all()) == 173
    assert repr(mr.Skill("SP")) == 'Skill("SP")'


@pytest.mark.parametrize(
    "make",
    [
        lambda: mr.Skill("連激"),  # pyright: ignore[reportArgumentType]  知らない名前 (型チェックでも弾かれる)
        lambda: mr.Skill("ＫＯ"),  # pyright: ignore[reportArgumentType]  全角は受け付けない
        lambda: mr.Skill.from_id(137),  # お守りに付かないスキル
        lambda: mr.Charm(mr.Skill("連撃"), 4, mr.Skill("痛撃")),  # pts2 がない
        lambda: mr.Charm(mr.Skill("連撃"), 4, pts2=5),  # skill2 がない
        lambda: mr.Query(mr.Melding.Juju, WANT, kind=mr.Kind.Shining),  # マカフシギ錬金では出ない
        lambda: mr.Query(mr.Melding.Juju, mr.Charm(mr.Skill("攻撃"), 1)),  # どの種類でも出ない
        lambda: mr.Query(mr.Melding.Juju, WANT, rank=5),
        lambda: mr.outcomes(mr.Melding.Juju, 0, 10, rank=0),
        lambda: mr.search(mr.Query(mr.Melding.Juju, WANT), 0, 10, nthreads=0),
    ],
)
def test_invalid_arguments_are_value_errors(make):
    with pytest.raises(ValueError):
        make()


def test_outcomes():
    os = mr.outcomes(mr.Melding.Juju, 4722, 4725)
    assert [o.frame for o in os] == [4722, 4723, 4724]
    assert os[1].charms[0] == (mr.Kind.Weathered, WANT)
    assert str(os[1].game_time()) == "0d 0h 2m 37s 13f"
    assert mr.outcomes(mr.Melding.Juju, 10, 10) == []


def test_skill_info():
    info = mr.skill_info(mr.Melding.Juju)
    assert len(info) == 134
    guard = next(i for i in info if i.skill == mr.Skill("ガ性"))
    assert guard.pts1 == (1, 7)  # 風化の 3〜7 と古びたの 1〜6 を合わせたもの
    assert repr(guard).startswith('SkillInfo(skill=Skill("ガ性"), pts1=(1, 7), pts2=')
    assert [i.skill.id for i in info] == sorted(i.skill.id for i in info)


def test_search_meldings():
    # 4723 で錬金して出るお守りを全部 (何個目の指定あり) 渡すと、4723 が見つかる
    made = mr.outcomes(mr.Melding.Juju, 4723, 4724)[0].charms
    targets = [mr.MeldingTarget(mr.Query(mr.Melding.Juju, c), j + 1) for j, (_, c) in enumerate(made)]
    found = mr.search_meldings(targets, 0, 1_000_000)
    hit = next(m for m in found if m.frame == 4723)
    assert hit.positions == list(range(1, len(made) + 1))
    assert str(hit.game_time()) == "0d 0h 2m 37s 13f"
    key = lambda fs: [(m.frame, m.indexes, m.positions) for m in fs]
    assert key(mr.search_meldings(targets, 0, 1_000_000, nthreads=3)) == key(found)
    # 何個目の範囲外、錬金のやり方が揃っていない
    q = targets[0].query
    with pytest.raises(ValueError):
        mr.search_meldings([mr.MeldingTarget(q, 5)], 0, 10)
    worn = mr.Charm(mr.Skill("火攻"), 1)
    halcyon = mr.MeldingTarget(mr.Query(mr.Melding.Halcyon, worn))
    with pytest.raises(ValueError):
        mr.search_meldings([targets[0], halcyon], 0, 10)


def test_game_time():
    t = mr.GameTime(2_592_000 * 3 + 108_000 * 5 + 1800 * 7 + 30 * 9 + 11)
    assert (t.days, t.hours, t.minutes, t.seconds, t.frames) == (3, 5, 7, 9, 11)
    assert str(t) == "3d 5h 7m 9s 11f"
    assert mr.GameTime(4723).to_timedelta() == datetime.timedelta(seconds=157, microseconds=433333)
    assert mr.GameTime(30) < mr.GameTime(31)


def test_reprs():
    q = mr.Query(mr.Melding.Juju, WANT)
    assert repr(q) == (
        'Query(melding=Melding.Juju, charm=Charm(skill1=Skill("連撃"), pts1=4, skill2=Skill("痛撃"), pts2=5, slots=3), '
        "kind=None, rank=4)"
    )
    assert repr(mr.search(q, 4723, 4724)[0]).startswith("Found(position=4723, kind=Kind.Weathered, charm=Charm(")
    assert repr(mr.GameTime(4723)) == "GameTime(4723)"


def test_charm_str():
    # 本家の charmpush と同じ形。1 文字の名前と KO・SP は全角スペースで 2 文字の幅に揃える
    assert str(WANT) == "連撃  4 痛撃   5 S3"
    assert str(mr.Charm(skill1=mr.Skill("特会"), pts1=6, slots=3)) == "特会  6 -------- S3"
    ko = mr.Charm(skill1=mr.Skill("KO"), pts1=10, skill2=mr.Skill("毒"), pts2=-3, slots=0)
    assert str(ko) == "KO　 10 毒　  -3 S0"


def test_outcome_str():
    # 本家の jujupush と同じく 1 個 1 行。種類の名前は全角 7 文字の幅に揃える
    assert str(mr.Kind.Worn) == "古びたお守り"
    assert str(mr.outcomes(mr.Melding.Juju, 4713, 4714)[0]) == (
        "風化したお守り  回量  5 KO　   5 S0\n"
        "風化したお守り  重強  2 真紫   3 S0\n"
        "古びたお守り　  特会  1 白疾   1 S2"
    )
