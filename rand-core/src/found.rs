//! 検索で見つけた 1 件と、それを錬金のフレームに直す処理 (本家の aimpoint_melding)。
//!
//! ライセンスは [`crate::rng`] の記載を参照 (apmnnn/mhxx-rng, MIT)。

use std::sync::Arc;

use crate::charm::{Charm, Kind, charm_at, state};
use crate::melding::kinds;
use crate::query::Query;
use crate::rng::ascend;
use crate::time::GameTime;

/// 見つけた位置に届く錬金のフレームと、そのお守りが何個目か
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Aimpoint {
    /// 錬金したフレーム
    pub start: u64,
    /// 何個目か (1 から)
    pub index: u8,
}

impl Aimpoint {
    /// start までの時間 (乱数は 1 フレームに 1 つ進むので、ゲームを始めてから錬金するまでの時間)
    pub fn game_time(&self) -> GameTime {
        GameTime(self.start)
    }
}

/// 検索で見つけた 1 件
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// そのお守りを作る乱数の始まり
    pub position: u64,
    /// 探した Query の何番目で見つかったか (search_many に渡した順。search では 0)
    pub index: usize,
    /// どの種類として見つかったか (Query の種類が決まっていなければ、出うる種類のどれか)
    pub kind: Kind,
    /// 探した Query (同じ Query で見つけた件どうしで共有する)
    pub(crate) query: Arc<Query>,
}

impl Found {
    /// 探した Query
    pub fn query(&self) -> &Query {
        &self.query
    }

    /// 鑑定画面に出るお守りの中身。AtLeast で探したときは、ポイントやスロットが探したものより大きいことがある
    pub fn charm(&self) -> Charm {
        charm_at(self.kind, self.position).shown()
    }

    /// この位置に、kind の種類のお守りとして届く錬金のフレームと何個目か (start の小さい順。本家の aimpoint_melding)。
    /// 錬金のやり方とランクは Query のもの。錬金のフレームは見つけた位置より最大 18 (天運の錬金は 30) 手前になる
    pub fn aimpoints(&self) -> Vec<Aimpoint> {
        let q = &self.query;
        let p = self.position;
        // 錬金したフレーム f を p の手前から試す。f の 1 つ前の乱数で出る種類の並びが決まり、f から順に作る
        let lo = p.saturating_sub(q.melding.max_offset());
        let v = Values::new(lo as i64 - 1, (p - lo) as usize + 4);
        let mut out = Vec::new();
        for f in lo..=p {
            let mut pos = f;
            for (j, k) in kinds(q.melding, q.rank, v.at(f as i64 - 1))
                .into_iter()
                .enumerate()
            {
                if pos >= p {
                    if pos == p && k == self.kind {
                        out.push(Aimpoint {
                            start: f,
                            index: j as u8 + 1,
                        });
                    }
                    break;
                }
                // お守り 1 個で乱数を 4 個 (第 2 スキルなし) か 6 個 (あり) 使う
                pos += if v.at(pos as i64 + 2) % 100 < k.table().no_skill2 {
                    4
                } else {
                    6
                };
            }
        }
        out
    }
}

/// 位置 from (-1 もよい) から続く乱数
struct Values {
    from: i64,
    v: Vec<u32>,
}

impl Values {
    fn new(from: i64, n: usize) -> Self {
        let mut s = state(from);
        let v = (0..n)
            .map(|_| {
                let w = s[3];
                s = ascend(s);
                w
            })
            .collect();
        Self { from, v }
    }

    fn at(&self, pos: i64) -> u32 {
        self.v[(pos - self.from) as usize]
    }
}
