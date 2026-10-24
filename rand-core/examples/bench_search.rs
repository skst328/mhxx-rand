//! search の速さを測る (cargo run --release --example bench_search)

use std::time::Instant;

use mhxx_rand::{Charm, Compare, Kind, Melding, Query, Rank, Skill, search};

fn main() {
    let n = 100_000_000;
    let cases = [
        (
            "連撃4 痛撃5 S3",
            Charm {
                skill1: Skill::ChainCrit,
                pts1: 4,
                skill2: Some((Skill::Tenderizer, 5)),
                slots: 3,
            },
        ),
        (
            "斬味5 - S1",
            Charm {
                skill1: Skill::Sharpness,
                pts1: 5,
                skill2: None,
                slots: 1,
            },
        ),
    ];
    for (name, c) in cases {
        let q = Query::new(Melding::Juju, Rank::MAX, Some(Kind::Weathered), &c).unwrap();
        // 3 回測って一番短いもの
        let mut t = f64::MAX;
        let mut found = 0;
        for _ in 0..3 {
            let s = Instant::now();
            found = search(&q, Compare::Exact, 0..n).len();
            t = t.min(s.elapsed().as_secs_f64());
        }
        println!("{name:<16} {found:>6} 件  {t:.3}s ({n} フレーム)");
    }
}
