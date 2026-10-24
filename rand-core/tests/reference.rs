//! 本家 (reference/mhxx_rng.py) と同じ結果になるか。正解は reference/fixtures.py が書き出した
//! tests/fixtures/reference.json (お守りは [第 1 スキル, ポイント, 第 2 スキル (なければ -1), ポイント, スロット])

use std::sync::OnceLock;

use mhxx_rand::{
    Charm, ComboQuery, Compare, Diagnose, Kind, Melding, Query, Rank, Skill, charm_at, outcomes,
    search, search_combo,
};
use serde_json::Value;

fn fixture() -> &'static Value {
    static DATA: OnceLock<Value> = OnceLock::new();
    DATA.get_or_init(|| {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reference.json");
        serde_json::from_str(&std::fs::read_to_string(path).expect(path)).unwrap()
    })
}

fn kind(v: &Value) -> Kind {
    match v.as_str().unwrap() {
        "weathered" => Kind::Weathered,
        "worn" => Kind::Worn,
        "shining" => Kind::Shining,
        "mystery" => Kind::Mystery,
        k => panic!("知らない種類 {k}"),
    }
}

fn melding(v: &Value) -> Melding {
    match v.as_str().unwrap() {
        "juju" => Melding::Juju,
        "halcyon" => Melding::Halcyon,
        m => panic!("知らない錬金 {m}"),
    }
}

fn rank(v: &Value) -> Rank {
    Rank::new(v.as_u64().unwrap() as u8).unwrap()
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

fn list<'a>(key: &str) -> &'a [Value] {
    fixture()[key].as_array().unwrap()
}

/// 位置ごとに、その位置から作るお守りが同じ (4 種類 x 始まりの近く・乱数で選んだ位置・10 億の手前)
#[test]
fn charms_match() {
    for case in list("charms") {
        let p = case["position"].as_u64().unwrap();
        let got = charm_at(kind(&case["kind"]), p).shown();
        assert_eq!(got, charm(&case["charm"]), "{case}");
    }
}

/// 錬金したフレームごとに、出るお守りが同じ (マカフシギ・天運 x ランク 1〜4)
#[test]
fn meldings_match() {
    for case in list("meldings") {
        let frame = case["frame"].as_u64().unwrap();
        let got = &outcomes(
            melding(&case["melding"]),
            rank(&case["rank"]),
            frame..frame + 1,
        )[0];
        let want: Vec<(Kind, Charm)> = case["charms"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| (kind(&c["kind"]), charm(&c["charm"])))
            .collect();
        assert_eq!(got.charms, want, "{case}");
    }
}

/// 位置ごとに、そこに届く錬金のフレームが同じ (本家の当たりフレーム。位置 0 より前は本家の側で外してある)
#[test]
fn aimpoints_match() {
    for case in list("aimpoints") {
        let (k, p) = (kind(&case["kind"]), case["position"].as_u64().unwrap());
        // 位置 p のお守りそのものを探せば、p で見つかる
        let q = Query::new(
            melding(&case["melding"]),
            rank(&case["rank"]),
            Some(k),
            &charm_at(k, p).shown(),
        )
        .unwrap();
        let found = search(&q, Compare::Exact, p..p + 1);
        assert_eq!(found.len(), 1, "{case}");
        let got: Vec<u64> = found[0].aimpoints().iter().map(|a| a.start).collect();
        let want: Vec<u64> = case["starts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_u64().unwrap())
            .collect();
        assert_eq!(got, want, "{case}");
    }
}

/// お守りの検索が、本家の numba 版 (show_fast) と同じ位置を見つける。見つかる数が少ないものは 10 億フレーム
#[test]
fn searches_match() {
    check_searches("searches");
}

/// どのスキルも最低 1 回は検索にかける: 4 種類の表の、第 1 スキルの全部の行と第 2 スキルの全部の行
#[test]
fn every_skill_searches_match() {
    check_searches("every_skill_searches");
}

/// 4 種類の表が、本家の set_blue などの値と同じ
#[test]
fn tables_match() {
    let pairs = |v: &[(u8, u8)]| {
        v.iter()
            .map(|&(a, b)| vec![a as u64, b as u64])
            .collect::<Vec<_>>()
    };
    let ids = |v: &[Skill]| v.iter().map(|s| s.id() as u64).collect::<Vec<_>>();
    let nums = |v: &Value| -> Vec<Vec<u64>> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|r| {
                r.as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_u64().unwrap())
                    .collect()
            })
            .collect()
    };
    for (name, want) in fixture()["tables"].as_object().unwrap() {
        let t = kind(&Value::from(name.as_str())).table();
        let flat = |v: &Value| {
            v.as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_u64().unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(t.skill1), flat(&want["skill1"]), "{name} skill1");
        assert_eq!(pairs(t.pts1), nums(&want["pts1"]), "{name} pts1");
        assert_eq!(ids(t.skill2), flat(&want["skill2"]), "{name} skill2");
        assert_eq!(pairs(t.pts2), nums(&want["pts2"]), "{name} pts2");
        let slots: Vec<Vec<u64>> = t
            .slots
            .iter()
            .map(|&(a, b, c)| vec![a as u64, b as u64, c as u64])
            .collect();
        assert_eq!(slots, nums(&want["slots"]), "{name} slots");
        assert_eq!(
            u64::from(t.no_skill2),
            want["no_skill2"].as_u64().unwrap(),
            "{name} no_skill2"
        );
    }
}

/// スキルの名前が本家の略称と同じ (全角スペースを取り、ＫＯ・ＳＰを半角にしたもの)。
/// Skill はお守りに付きうるもの (4 種類の表のどれかに出てくるもの) をちょうど全部持つ
#[test]
fn skills_match() {
    let names = fixture()["skill_names"].as_array().unwrap();
    for s in Skill::ALL {
        assert_eq!(s.name(), names[s.id() as usize].as_str().unwrap(), "{s:?}");
    }
    let mut in_tables: Vec<u16> = [Kind::Weathered, Kind::Worn, Kind::Shining, Kind::Mystery]
        .iter()
        .flat_map(|k| k.table().skill1.iter().chain(k.table().skill2))
        .map(|s| s.id())
        .collect();
    in_tables.sort();
    in_tables.dedup();
    assert_eq!(Skill::ALL.map(|s| s.id()).to_vec(), in_tables);
}

/// 調合の検索が、本家の search_combo と同じフレームを見つける (調合回数はいろいろ、最後が 99 に張り付いたものも)
#[test]
fn combos_match() {
    for case in list("combos") {
        let cumulative: Vec<Option<u8>> = case["cumulative"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| Some(v.as_u64().unwrap() as u8))
            .collect();
        let q = ComboQuery::new(&cumulative).unwrap();
        // 本家は 0 から step 個の乱数を読む。差分の先頭が s の一致の報告値は s + shift で、照合は s から span 先まで
        let step = case["step"].as_i64().unwrap();
        let used = cumulative.len() - 3 - usize::from(cumulative.last() == Some(&Some(99)));
        assert_eq!(q.pattern_len(), used - 1);
        let shift = -30 + 2 * (cumulative.len() as i64 - 1);
        let span = 5 * (q.pattern_len() as i64 - 1);
        let end = (step - span + shift).max(0) as u64;
        let got = search_combo(&q, 0..end, Diagnose::Never, None).frames;
        let want: Vec<u64> = case["frames"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f.as_u64().unwrap())
            .collect();
        assert_eq!(got, want, "{:?}", case["cumulative"]);
    }
}

fn check_searches(key: &str) {
    for case in list(key) {
        let k = kind(&case["kind"]);
        let m = if matches!(k, Kind::Weathered | Kind::Worn) {
            Melding::Juju
        } else {
            Melding::Halcyon
        };
        let q = Query::new(m, Rank::MAX, Some(k), &charm(&case["charm"])).unwrap();
        let compare = if case["compare"] == "exact" {
            Compare::Exact
        } else {
            Compare::AtLeast
        };
        let (start, end) = (
            case["start"].as_u64().unwrap(),
            case["end"].as_u64().unwrap(),
        );
        let got: Vec<u64> = search(&q, compare, start..end)
            .iter()
            .map(|f| f.position)
            .collect();
        let want: Vec<u64> = case["positions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_u64().unwrap())
            .collect();
        assert!(
            !want.is_empty(),
            "見つからないケースは確かめにならない: {case}"
        );
        assert_eq!(
            got, want,
            "{:?} {} {start}..{end}",
            case["charm"], case["compare"]
        );
    }
}
