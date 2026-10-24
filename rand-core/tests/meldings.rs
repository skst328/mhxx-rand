//! 何個ものお守りから錬金のフレームを探す search_meldings。
//!
//! 正解の tests/fixtures/renkin/ は、mhxx-renkin-reader の tests/fixtures/rng/ から写したもの
//! (experimental-charm-reader の frames.py が、本家の関数で出したもの。マカフシギ錬金・ランク 4)。
//! お守りは [第 1 スキル, ポイント, 第 2 スキル (なければ -1), ポイント, スロット]

use mhxx_rand::{
    Charm, FrameMatch, Kind, Melding, MeldingError, MeldingTarget, Outcome, Query, Rank, Skill,
    outcomes, search_meldings,
};
use serde_json::Value;

fn fixture(name: &str) -> Value {
    let path = format!(
        "{}/tests/fixtures/renkin/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(&path).expect(&path)).unwrap()
}

fn charm(v: &Value) -> Charm {
    let n = |i: usize| v[i].as_i64().unwrap();
    let skill = |id: i64| Skill::from_id(id as u16).unwrap();
    Charm {
        skill1: skill(n(0)),
        pts1: n(1) as i8,
        skill2: (n(2) >= 0).then(|| (skill(n(2)), n(3) as i8)),
        slots: n(4) as u8,
    }
}

/// 種類を決めない、マカフシギ錬金・ランク 4 のお守り 1 個
fn target(c: &Charm, index: Option<u8>) -> MeldingTarget {
    MeldingTarget {
        query: Query::new(Melding::Juju, Rank::MAX, None, c).unwrap(),
        index,
    }
}

/// お守り 1 個で探して、(フレーム, 何個目の一覧)
fn search_one(c: &Charm, index: Option<u8>, start: u64, end: u64) -> Vec<(u64, Vec<u8>)> {
    search_meldings(&[target(c, index)], start..end)
        .unwrap()
        .into_iter()
        .map(|m| {
            assert_eq!(m.positions, [m.indexes[0][0]], "{m:?}");
            (m.frame, m.indexes.into_iter().next().unwrap())
        })
        .collect()
}

/// お守り 1 個と何個目で探したものが、frames.py (本家の関数) と同じ
#[test]
fn single_charm_matches_python() {
    for case in fixture("melding.json")["searches"].as_array().unwrap() {
        let c = charm(&case["charm"]);
        let index = case["index"].as_u64().map(|i| i as u8);
        let (start, end) = (
            case["start"].as_u64().unwrap(),
            case["end"].as_u64().unwrap(),
        );
        let want: Vec<(u64, Vec<u8>)> = case["found"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| {
                (
                    f[0].as_u64().unwrap(),
                    f[1].as_array()
                        .unwrap()
                        .iter()
                        .map(|i| i.as_u64().unwrap() as u8)
                        .collect(),
                )
            })
            .collect();
        // マカフシギ錬金で出ないお守りは、Query を作る時点で弾く (frames.py は空を返す)
        let Ok(q) = Query::new(Melding::Juju, Rank::MAX, None, &c) else {
            assert!(want.is_empty(), "{case}");
            continue;
        };
        let got: Vec<(u64, Vec<u8>)> =
            search_meldings(&[MeldingTarget { query: q, index }], start..end)
                .unwrap()
                .into_iter()
                .map(|m| (m.frame, m.indexes.into_iter().next().unwrap()))
                .collect();
        assert_eq!(got, want, "{c:?} 何個目 {index:?}");
    }
}

/// 本家の「有名なお守り」を 0〜10 億フレームで探したものが、frames.py と同じ (見つかる数が少ない 2 件)
#[test]
fn famous_charms_in_a_billion_frames() {
    for case in fixture("famous.json").as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        if !["連撃5 達人10 S3", "貫強6 達人10 S3"].contains(&name) {
            continue;
        }
        let want: Vec<u64> = case["frames"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f.as_u64().unwrap())
            .collect();
        let got: Vec<u64> = search_one(&charm(&case["charm"]), None, 0, 1_000_000_000)
            .iter()
            .map(|(f, _)| *f)
            .collect();
        assert_eq!(got, want, "{name}");
    }
}

/// 範囲の錬金の結果 all を全部見て、全部のお守りが別の位置に出るかを確かめたもの (総当たり)
fn brute(all: &[Outcome], targets: &[MeldingTarget]) -> Vec<(u64, Vec<Vec<u8>>)> {
    all.iter()
        .filter_map(|o| {
            let indexes: Vec<Vec<u8>> = targets
                .iter()
                .map(|t| {
                    (0..o.charms.len())
                        .filter(|&j| {
                            let (k, c) = &o.charms[j];
                            t.index.is_none_or(|i| j + 1 == i as usize)
                                && t.query.kinds().any(|qk| qk == *k)
                                && c == t.query.charm()
                        })
                        .map(|j| j as u8 + 1)
                        .collect()
                })
                .collect();
            distinct(&indexes).then_some((o.frame, indexes))
        })
        .collect()
}

/// 全部が別の位置になる選び方があるか
fn distinct(options: &[Vec<u8>]) -> bool {
    fn pick(options: &[Vec<u8>], chosen: &mut Vec<u8>) -> bool {
        let Some((first, rest)) = options.split_first() else {
            return true;
        };
        first.iter().any(|&p| {
            if chosen.contains(&p) {
                return false;
            }
            chosen.push(p);
            let ok = pick(rest, chosen);
            chosen.pop();
            ok
        })
    }
    pick(options, &mut Vec::new())
}

/// 見つけた 1 件が、そのフレームの錬金を作り直したものと合う: 位置の選び方がそれぞれの出る位置で、全部が別
fn assert_consistent(m: &FrameMatch) {
    assert!(
        m.positions
            .iter()
            .zip(&m.indexes)
            .all(|(p, ix)| ix.contains(p)),
        "{m:?}"
    );
    assert!(
        (1..m.positions.len()).all(|i| !m.positions[..i].contains(&m.positions[i])),
        "{m:?}"
    );
}

/// 何個ものお守りで探したものが総当たりと同じ (同じ錬金の組・ほかの錬金との組・同じ中身を 2 個・何個目の指定
/// あり・なし・種類を決めたもの)。マカフシギと天運、範囲は 0 からと 10 億の手前
#[test]
fn many_charms_match_brute_force() {
    for m in [Melding::Juju, Melding::Halcyon] {
        for start in [0u64, 999_970_000] {
            let frames = start..start + 30_000;
            let all = outcomes(m, Rank::MAX, frames.clone());
            // f は start からの何フレーム目か、j は何個目 (0 から)
            let pick = |f: usize, j: usize, index: bool, kind: bool| {
                let (k, c) = all[f].charms[j];
                MeldingTarget {
                    query: Query::new(m, Rank::MAX, kind.then_some(k), &c).unwrap(),
                    index: index.then_some(j as u8 + 1),
                }
            };
            // (お守り, 全部をそのまま取り出した錬金のフレーム。別々の錬金の組なら None)
            let cases: Vec<(Vec<MeldingTarget>, Option<usize>)> = vec![
                (
                    vec![pick(17, 0, true, false), pick(17, 1, true, false)],
                    Some(17),
                ),
                (
                    vec![pick(17, 0, false, false), pick(17, 1, false, true)],
                    Some(17),
                ),
                (
                    vec![pick(4321, 1, false, false), pick(4321, 0, true, false)],
                    Some(4321),
                ),
                (
                    vec![pick(29_000, 0, false, false), pick(123, 1, false, false)],
                    None,
                ),
                (
                    vec![pick(500, 0, false, false), pick(500, 0, false, false)],
                    None,
                ),
                (
                    (0..all[2000].charms.len())
                        .map(|j| pick(2000, j, false, false))
                        .collect(),
                    Some(2000),
                ),
            ];
            for (targets, source) in &cases {
                let got = search_meldings(targets, frames.clone()).unwrap();
                got.iter().for_each(assert_consistent);
                let got: Vec<(u64, Vec<Vec<u8>>)> =
                    got.into_iter().map(|x| (x.frame, x.indexes)).collect();
                assert_eq!(got, brute(&all, targets), "{m:?} {frames:?} {targets:?}");
                if let Some(f) = source {
                    assert!(
                        got.iter().any(|(g, _)| *g == start + *f as u64),
                        "取り出したフレームが見つからない"
                    );
                }
            }
        }
    }
}

/// 範囲を区切って探しても、通して探したものと同じ
#[test]
fn split_ranges_give_same_result() {
    let all = outcomes(Melding::Juju, Rank::MAX, 0..10);
    let targets = [target(&all[3].charms[0].1, None)];
    let whole = search_meldings(&targets, 0..300_000).unwrap();
    assert!(!whole.is_empty());
    for cut in [0, 1, 3, 4, 123_457, 300_000] {
        let mut parts = search_meldings(&targets, 0..cut).unwrap();
        parts.extend(search_meldings(&targets, cut..300_000).unwrap());
        assert_eq!(parts, whole, "区切り {cut}");
    }
}

/// 0〜10 億フレームを通しで、10 億の手前で錬金したお守りを全部 (何個目の指定あり) 渡すと、そのフレームが見つかる。
/// 範囲全体を流すのは、お守りの数によらず 1 回
#[test]
fn many_charms_in_a_billion_frames() {
    let frame = 999_999_000;
    let made = &outcomes(Melding::Juju, Rank::MAX, frame..frame + 1)[0].charms;
    let targets: Vec<MeldingTarget> = made
        .iter()
        .enumerate()
        .map(|(j, (_, c))| target(c, Some(j as u8 + 1)))
        .collect();
    let got = search_meldings(&targets, 0..1_000_000_000).unwrap();
    let hit = got.iter().find(|m| m.frame == frame).expect("見つからない");
    assert_eq!(hit.positions, (1..=made.len() as u8).collect::<Vec<_>>());
    got.iter().for_each(assert_consistent);
}

/// 何個目がその錬金の最大の数を超える、錬金のやり方かランクが揃っていない、はエラー。空なら空
#[test]
fn invalid_targets_are_errors() {
    let c = Charm {
        skill1: Skill::Vault,
        pts1: 4,
        skill2: None,
        slots: 1,
    }; // 跳躍4 S1
    for index in [0, 5, 12] {
        let t = [target(&c, Some(1)), target(&c, Some(index))];
        assert_eq!(
            search_meldings(&t, 0..1000),
            Err(MeldingError::Index { index })
        );
    }
    // 天運の錬金は 6 個まで出る
    let shining = Charm {
        skill1: Skill::Attack,
        pts1: 1,
        skill2: None,
        slots: 0,
    };
    let h = |index| MeldingTarget {
        query: Query::new(Melding::Halcyon, Rank::MAX, None, &shining).unwrap(),
        index,
    };
    assert!(search_meldings(&[h(Some(6))], 0..1000).is_ok());
    assert_eq!(
        search_meldings(&[h(Some(7))], 0..1000),
        Err(MeldingError::Index { index: 7 })
    );
    // マカフシギと天運、ランク違いは混ぜられない
    let worn = Charm {
        skill1: Skill::FireAtk,
        pts1: 1,
        skill2: None,
        slots: 0,
    };
    let juju = MeldingTarget {
        query: Query::new(Melding::Juju, Rank::MAX, Some(Kind::Worn), &worn).unwrap(),
        index: None,
    };
    let halcyon = MeldingTarget {
        query: Query::new(Melding::Halcyon, Rank::MAX, Some(Kind::Worn), &worn).unwrap(),
        index: None,
    };
    let rank2 = MeldingTarget {
        query: Query::new(Melding::Juju, Rank::new(2).unwrap(), None, &worn).unwrap(),
        index: None,
    };
    assert_eq!(
        search_meldings(&[juju.clone(), halcyon], 0..1000),
        Err(MeldingError::Mixed)
    );
    assert_eq!(
        search_meldings(&[juju.clone(), rank2], 0..1000),
        Err(MeldingError::Mixed)
    );
    assert_eq!(search_meldings(&[], 0..1000), Ok(Vec::new()));
    assert_eq!(search_meldings(&[juju], 10..10), Ok(Vec::new()));
}
