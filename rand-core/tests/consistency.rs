//! 内側どうしで食い違いがないか (本家の正解は使わない)

use std::collections::BTreeSet;

use mhxx_rand::rng::{ascend, jump};
use mhxx_rand::{
    Charm, Compare, Kind, Melding, Outcome, Query, QueryError, Rank, Reason, Skill, make, outcomes,
    search, search_many, skill_info,
};

fn charm(s1: &str, p1: i8, skill2: Option<(&str, i8)>, slots: u8) -> Charm {
    let s = |n: &str| Skill::from_name(n).unwrap();
    Charm {
        skill1: s(s1),
        pts1: p1,
        skill2: skill2.map(|(n, p)| (s(n), p)),
        slots,
    }
}

/// c が want 以上か (search の AtLeast と同じ決まりを、素直に書いたもの)
fn at_least(c: &Charm, want: &Charm) -> bool {
    c.skill1 == want.skill1
        && c.pts1 >= want.pts1
        && c.slots >= want.slots
        && want
            .skill2
            .is_none_or(|(s, p)| c.skill2.is_some_and(|(cs, cp)| cs == s && cp >= p))
}

/// 確かめる Query: よく出るもの・まれなもの・第 2 スキルなし・種類を省略したもの
fn queries() -> Vec<(Query, Compare)> {
    let q = |m, kind, c| Query::new(m, Rank::MAX, kind, &c).unwrap();
    vec![
        (
            q(
                Melding::Juju,
                Some(Kind::Weathered),
                charm("斬味", 5, None, 1),
            ),
            Compare::Exact,
        ),
        (
            q(
                Melding::Juju,
                Some(Kind::Weathered),
                charm("斬味", 4, Some(("攻撃", 5)), 1),
            ),
            Compare::AtLeast,
        ),
        (
            q(Melding::Juju, Some(Kind::Worn), charm("火攻", 2, None, 0)),
            Compare::AtLeast,
        ),
        (
            q(Melding::Juju, None, charm("聴覚", 3, None, 0)),
            Compare::Exact,
        ),
        (
            q(Melding::Juju, None, charm("聴覚", 3, Some(("風圧", 3)), 0)),
            Compare::AtLeast,
        ),
        (
            q(
                Melding::Halcyon,
                Some(Kind::Shining),
                charm("攻撃", 1, None, 0),
            ),
            Compare::AtLeast,
        ),
        (
            q(Melding::Halcyon, None, charm("体力", 3, None, 0)),
            Compare::Exact,
        ),
    ]
}

/// search が、位置ごとにお守りを作って比べた総当たりと同じ (位置・種類)。10 億の手前の範囲も
#[test]
fn search_matches_brute_force() {
    for start in [0u64, 999_950_000] {
        let frames = start..start + 50_000;
        for (q, compare) in queries() {
            let got: Vec<(u64, Kind)> = search(&q, compare, frames.clone())
                .iter()
                .map(|f| (f.position, f.kind))
                .collect();
            // 乱数を 1 つずつ進めながら、位置ごとに 7 個の乱数でお守りを作る
            let mut s = jump(frames.start);
            let mut r: [u32; 7] = std::array::from_fn(|_| {
                let w = s[3];
                s = ascend(s);
                w
            });
            let mut want = Vec::new();
            for p in frames.clone() {
                for k in q.kinds() {
                    let c = make(k, &r).shown();
                    let hit = match compare {
                        Compare::Exact => c == *q.charm(),
                        Compare::AtLeast => at_least(&c, q.charm()),
                    };
                    if hit {
                        want.push((p, k));
                    }
                }
                r = [r[1], r[2], r[3], r[4], r[5], r[6], s[3]];
                s = ascend(s);
            }
            assert!(
                !want.is_empty(),
                "見つからないケースは確かめにならない: {q:?}"
            );
            assert_eq!(got, want, "{q:?} {compare:?} {frames:?}");
        }
    }
}

/// 当たりフレームと錬金の結果が行き来できる: 見つけた位置の当たりフレームで錬金すると、その何個目に同じお守りが出る。
/// 逆に、範囲の中で錬金して出るものは、全部どこかの位置の当たりフレームとして挙がる
#[test]
fn aimpoints_match_outcomes() {
    let frames = 1_000_000u64..1_200_000;
    // 錬金の結果は、錬金のやり方ごとに 1 回だけ作る (どの Query もランクは最高)
    let melded: Vec<(Melding, Vec<Outcome>)> = [Melding::Juju, Melding::Halcyon]
        .into_iter()
        .map(|m| (m, outcomes(m, Rank::MAX, frames.clone())))
        .collect();
    for (q, compare) in queries() {
        let matches = |k: Kind, c: &Charm| {
            q.kinds().any(|qk| qk == k)
                && match compare {
                    Compare::Exact => c == q.charm(),
                    Compare::AtLeast => at_least(c, q.charm()),
                }
        };
        // 錬金の結果から: (フレーム, 何個目)
        let want: BTreeSet<(u64, u8)> = melded
            .iter()
            .find(|(m, _)| *m == q.melding())
            .unwrap()
            .1
            .iter()
            .flat_map(|o| {
                o.charms
                    .iter()
                    .enumerate()
                    .filter(|(_, (k, c))| matches(*k, c))
                    .map(|(j, _)| (o.frame, j as u8 + 1))
            })
            .collect();
        // 検索から: 錬金のフレームが範囲に入るもの (位置は錬金のフレームより最大 30 後ろ)
        let mut got = BTreeSet::new();
        for f in search(&q, compare, frames.start..frames.end + 30) {
            for a in f.aimpoints() {
                let o = &outcomes(q.melding(), q.rank(), a.start..a.start + 1)[0];
                assert_eq!(
                    o.charms[a.index as usize - 1],
                    (f.kind, f.charm()),
                    "{f:?} {a:?}"
                );
                if frames.contains(&a.start) {
                    got.insert((a.start, a.index));
                }
            }
        }
        assert!(
            !want.is_empty(),
            "見つからないケースは確かめにならない: {q:?}"
        );
        assert_eq!(got, want, "{q:?} {compare:?}");
    }
}

/// 種類を省略した Query は、出うる種類ごとの Query を合わせたものと同じ
#[test]
fn any_kind_is_union_of_kinds() {
    for (q, compare) in queries().into_iter().filter(|(q, _)| q.kind().is_none()) {
        let frames = 0..3_000_000;
        let got: Vec<(u64, Kind)> = search(&q, compare, frames.clone())
            .iter()
            .map(|f| (f.position, f.kind))
            .collect();
        let mut want: Vec<(u64, Kind)> = q
            .melding()
            .kinds()
            .iter()
            .filter_map(|&k| Query::new(q.melding(), q.rank(), Some(k), q.charm()).ok())
            .flat_map(|qk| {
                search(&qk, compare, frames.clone())
                    .into_iter()
                    .map(|f| (f.position, f.kind))
            })
            .collect();
        want.sort_by_key(|&(p, k)| (p, k as u8));
        assert_eq!(got, want, "{q:?}");
    }
}

/// search_many は、Query ごとの search を位置の順に合わせたものと同じ (種類や錬金のやり方が混ざっていても)
#[test]
fn search_many_matches_search() {
    let qs = queries();
    for compare in [Compare::Exact, Compare::AtLeast] {
        let all: Vec<Query> = qs.iter().map(|(q, _)| q.clone()).collect();
        let frames = 0..3_000_000;
        let got: Vec<(u64, usize, Kind)> = search_many(&all, compare, frames.clone())
            .iter()
            .map(|f| (f.position, f.index, f.kind))
            .collect();
        let mut want: Vec<(u64, usize, Kind)> = all
            .iter()
            .enumerate()
            .flat_map(|(i, q)| {
                search(q, compare, frames.clone())
                    .into_iter()
                    .map(move |f| (f.position, i, f.kind))
            })
            .collect();
        want.sort_by_key(|&(p, i, k)| (p, i, k as u8));
        assert_eq!(got, want, "{compare:?}");
    }
}

/// 範囲を区切って探しても、通して探したものと同じ (区切りの前後・長さ 0 と 1 も)
#[test]
fn split_ranges_give_same_result() {
    for (q, compare) in queries() {
        let whole: Vec<u64> = search(&q, compare, 0..200_000)
            .iter()
            .map(|f| f.position)
            .collect();
        for cut in [0, 1, 3, 77_777, 199_999, 200_000] {
            let mut parts: Vec<u64> = search(&q, compare, 0..cut)
                .iter()
                .map(|f| f.position)
                .collect();
            parts.extend(search(&q, compare, cut..200_000).iter().map(|f| f.position));
            assert_eq!(parts, whole, "{q:?} 区切り {cut}");
        }
        assert!(search(&q, compare, 10..10).is_empty());
    }
}

/// スキルごとのポイントの範囲: マカフシギ錬金のスキルは 134 個 (鑑定スキャンのスキルの一覧と同じ)。
/// 錬金で出たお守りのポイントは、どれも範囲に入る。範囲に入っていれば、その錬金の Query が作れる種類がある
#[test]
fn skill_info_covers_melded_charms() {
    let juju = skill_info(Melding::Juju);
    assert_eq!(juju.len(), 134);
    let guard = juju.iter().find(|i| i.skill == Skill::Guard).unwrap();
    // 風化したお守りの 3〜7 と古びたお守りの 1〜6 を合わせたもの
    assert_eq!(guard.pts1, Some(1..=7));
    for m in [Melding::Juju, Melding::Halcyon] {
        let info = skill_info(m);
        assert!(
            info.windows(2).all(|w| w[0].skill < w[1].skill),
            "番号の小さい順"
        );
        assert!(info.iter().all(|i| i.pts1.is_some() || i.pts2.is_some()));
        let find = |s: Skill| info.iter().find(|i| i.skill == s).unwrap();
        for o in outcomes(m, Rank::MAX, 0..20_000) {
            for (_, c) in &o.charms {
                assert!(
                    find(c.skill1)
                        .pts1
                        .as_ref()
                        .is_some_and(|r| r.contains(&c.pts1)),
                    "{c:?}"
                );
                if let Some((s2, p2)) = c.skill2 {
                    assert!(
                        find(s2).pts2.as_ref().is_some_and(|r| r.contains(&p2)),
                        "{c:?}"
                    );
                }
            }
        }
        // 範囲の両端で、どれかの種類の Query が作れる (選択肢に出したものは探せる)
        for i in &info {
            if let Some(r) = &i.pts1 {
                for p in [*r.start(), *r.end()] {
                    let c = Charm {
                        skill1: i.skill,
                        pts1: p,
                        skill2: None,
                        slots: 0,
                    };
                    assert!(Query::new(m, Rank::MAX, None, &c).is_ok(), "{i:?} {p}");
                }
            }
        }
    }
}

/// 出ないお守りは、Query を作る時点で理由付きの Err になる
#[test]
fn impossible_queries_are_errors() {
    let err = |m, kind, c: Charm| Query::new(m, Rank::MAX, kind, &c).unwrap_err();
    let one = |k, r| QueryError::Impossible(vec![(k, r)]);
    let w = Some(Kind::Weathered);
    // マカフシギ錬金では光るお守りは出ない
    assert_eq!(
        err(
            Melding::Juju,
            Some(Kind::Shining),
            charm("攻撃", 1, None, 0)
        ),
        one(Kind::Shining, Reason::NotInMelding)
    );
    // 攻撃は風化したお守りの第 1 スキルにならない
    assert_eq!(
        err(Melding::Juju, w, charm("攻撃", 1, None, 0)),
        one(Kind::Weathered, Reason::Skill1)
    );
    // 風化したお守りの斬味は 3〜7
    assert_eq!(
        err(Melding::Juju, w, charm("斬味", 8, None, 0)),
        one(Kind::Weathered, Reason::Pts1)
    );
    assert_eq!(
        err(Melding::Juju, w, charm("斬味", 5, None, 4)),
        one(Kind::Weathered, Reason::Slots)
    );
    assert_eq!(
        err(Melding::Juju, w, charm("斬味", 5, Some(("斬味", 3)), 0)),
        one(Kind::Weathered, Reason::SameSkill)
    );
    assert_eq!(
        err(Melding::Juju, w, charm("斬味", 5, Some(("攻撃", 11)), 0)),
        one(Kind::Weathered, Reason::Pts2)
    );
    assert_eq!(
        err(Melding::Juju, w, charm("斬味", 5, Some(("攻撃", 0)), 0)),
        one(Kind::Weathered, Reason::Pts2)
    );
    // なぞのお守りには第 2 スキルが付かない
    let mystery = charm("気絶", 3, Some(("毒", 3)), 0);
    assert_eq!(
        err(Melding::Halcyon, Some(Kind::Mystery), mystery),
        one(Kind::Mystery, Reason::Skill2)
    );
    // 種類を省略したら、出うる種類ごとの理由
    assert_eq!(
        err(Melding::Juju, None, charm("攻撃", 1, None, 0)),
        QueryError::Impossible(vec![
            (Kind::Weathered, Reason::Skill1),
            (Kind::Worn, Reason::Skill1)
        ])
    );
    // 片方の種類で出れば Err にならず、出る種類だけで探す
    let q = Query::new(Melding::Juju, Rank::MAX, None, &charm("斬味", 7, None, 0)).unwrap();
    assert_eq!(q.kinds().collect::<Vec<_>>(), [Kind::Weathered]);
}
