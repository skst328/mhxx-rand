//! 錬金で出るお守り (本家の meldingpush と juju_colors / halcyon_colors)。
//!
//! 錬金した位置 F ごとに:
//!   1. F の 1 つ前の乱数から、出るお守りの種類と個数を決める
//!   2. F から順にお守りを作る。1 個につき乱数を 4 個 (第 2 スキルなし) か 6 個 (あり) 使う
//!
//! ライセンスは [`crate::rng`] の記載を参照 (apmnnn/mhxx-rng, MIT)。

use std::collections::BTreeMap;
use std::ops::{Range, RangeInclusive};

use crate::Skill;
use crate::charm::{Charm, Generated, Kind, make, state};
use crate::rng::ascend;
use crate::time::GameTime;

/// 錬金のやり方
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Melding {
    /// マカフシギ錬金: 風化したお守りと古びたお守りが出る (本家の juju)
    Juju,
    /// 天運の錬金: 古びた・光る・なぞのお守りが出る (本家の halcyon)
    Halcyon,
}

impl Melding {
    /// 出うるお守りの種類
    pub const fn kinds(self) -> &'static [Kind] {
        match self {
            Self::Juju => &[Kind::Weathered, Kind::Worn],
            Self::Halcyon => &[Kind::Worn, Kind::Shining, Kind::Mystery],
        }
    }

    /// 1 回の錬金で出るお守りの最大の数 (最高ランク。マカフシギは風化 2 個 + 古びた 2 個、天運は 2〜6 個)
    pub const fn max_charms(self) -> u8 {
        match self {
            Self::Juju => 4,
            Self::Halcyon => 6,
        }
    }

    /// 錬金したフレームから、最後のお守りの最初の乱数までの最大の間 (お守り 1 個につき乱数を最大 6 個使う)
    pub(crate) const fn max_offset(self) -> u64 {
        (self.max_charms() as u64 - 1) * 6
    }
}

/// 投入する素材のランク (1〜4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rank(u8);

impl Rank {
    /// 最高ランク
    pub const MAX: Self = Self(4);

    /// 1〜4 でなければ None
    pub const fn new(n: u8) -> Option<Self> {
        if 1 <= n && n <= 4 {
            Some(Self(n))
        } else {
            None
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

/// 本家の rand: 下位 16 bit を初期値に、0 なら 1 にして x176 mod 65363 を n 回
fn rand(seed: u32, n: u32) -> u32 {
    let mut v = seed & 0xFFFF;
    for _ in 0..n {
        v = v.max(1) * 176 % 65363;
    }
    v
}

/// 錬金で出るお守りの種類の並び (seed は錬金の 1 つ前の位置の乱数)
pub fn kinds(melding: Melding, rank: Rank, seed: u32) -> Vec<Kind> {
    match melding {
        Melding::Juju => {
            // (風化の幅, 最小, 古びたの幅, 最小)
            let (w1, m1, w2, m2) = match rank.0 {
                1 => (2, 0, 1, 1),
                2 => (2, 0, 2, 1),
                3 => (2, 1, 2, 1),
                _ => (1, 2, 2, 1),
            };
            let n1 = rand(seed, 1) % w1 + m1;
            let n2 = rand(seed, 2) % w2 + m2;
            let mut v = vec![Kind::Weathered; n1 as usize];
            v.extend(std::iter::repeat_n(Kind::Worn, n2 as usize));
            v
        }
        Melding::Halcyon => {
            let (w, m) = match rank.0 {
                1 => (3, 1),
                2 => (4, 1),
                3 => (4, 2),
                _ => (5, 2),
            };
            let mut a = 1;
            let n = rand(seed, a) % w + m;
            a += 1;
            (0..n)
                .map(|_| {
                    while rand(seed, a) % 100 >= 85 {
                        a += 1;
                    }
                    let c = rand(seed, a) % 100;
                    a += 1;
                    if c < 10 {
                        Kind::Mystery
                    } else if c < 55 {
                        Kind::Shining
                    } else {
                        Kind::Worn
                    }
                })
                .collect()
        }
    }
}

/// 位置 frame で錬金したときに出るお守り (鑑定ボックスの並び順) と、その種類
pub fn meld(melding: Melding, rank: Rank, frame: u64) -> Vec<(Kind, Generated)> {
    // v[k] は位置 frame - 1 + k の乱数。足りなくなったら伸ばす
    let mut s = state(frame as i64 - 1);
    let mut v: Vec<u32> = Vec::new();
    meld_with(melding, rank, |k| {
        while v.len() <= k {
            v.push(s[3]);
            s = ascend(s);
        }
        v[k]
    })
}

/// [`meld`] を、手元の乱数から作る。v(d) は錬金したフレームの d - 1 個先の乱数 (v(0) が 1 つ前、v(1) が
/// 錬金したフレーム)。d は max_offset + 7 まで使う
pub(crate) fn meld_with(
    melding: Melding,
    rank: Rank,
    mut v: impl FnMut(usize) -> u32,
) -> Vec<(Kind, Generated)> {
    let mut pos = 1;
    kinds(melding, rank, v(0))
        .into_iter()
        .map(|kind| {
            let r: [u32; 7] = std::array::from_fn(|d| v(pos + d));
            let c = make(kind, &r);
            pos += if r[2] % 100 < kind.table().no_skill2 {
                4
            } else {
                6
            };
            (kind, c)
        })
        .collect()
}

/// 1 回の錬金の結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// 錬金したフレーム
    pub frame: u64,
    /// 出たお守り (鑑定ボックスの並び順) の種類と、鑑定画面に出る中身
    pub charms: Vec<(Kind, Charm)>,
}

impl Outcome {
    /// frame までの時間 (ゲームを始めてから錬金するまで)
    pub fn game_time(&self) -> GameTime {
        GameTime(self.frame)
    }
}

/// frames の範囲のフレームごとに、melding (rank) で錬金したときの結果 (フレームの小さい順)。
/// 狙ったフレームの前後で錬金したら何が出るかを見るのに使う (本家の around を錬金 1 回分に広げたもの)
pub fn outcomes(melding: Melding, rank: Rank, frames: Range<u64>) -> Vec<Outcome> {
    frames
        .map(|frame| Outcome {
            frame,
            charms: meld(melding, rank, frame)
                .into_iter()
                .map(|(k, c)| (k, c.shown()))
                .collect(),
        })
        .collect()
}

/// その錬金で出うるスキルと、第 1・第 2 スキルとしてのポイントの範囲
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillInfo {
    pub skill: Skill,
    /// そのスキルが第 1 スキルとして出るときの、ポイントの範囲 (なれなければ None)
    pub pts1: Option<RangeInclusive<i8>>,
    /// そのスキルが第 2 スキルとして出るときの、ポイントの範囲 (なれなければ None。錬金ではマイナスは出ないので 1 から)
    pub pts2: Option<RangeInclusive<i8>>,
}

/// melding で出うるスキル (番号の小さい順) と、ポイントの範囲。その錬金で出る種類の表を合わせたもの
/// (マカフシギ錬金なら風化・古びた)。画面でスキルとポイントの選択肢を作るのに使う
pub fn skill_info(melding: Melding) -> Vec<SkillInfo> {
    // 2 つの範囲を合わせる (片方が無ければもう片方)
    fn union(a: Option<RangeInclusive<i8>>, lo: u8, hi: u8) -> Option<RangeInclusive<i8>> {
        let (lo, hi) = (lo as i8, hi as i8);
        Some(a.map_or(lo..=hi, |r| *r.start().min(&lo)..=*r.end().max(&hi)))
    }
    let mut info: BTreeMap<Skill, SkillInfo> = BTreeMap::new();
    let new = |skill| SkillInfo {
        skill,
        pts1: None,
        pts2: None,
    };
    for kind in melding.kinds() {
        let t = kind.table();
        for (&s, &(lo, hi)) in t.skill1.iter().zip(t.pts1) {
            let i = info.entry(s).or_insert_with(|| new(s));
            i.pts1 = union(i.pts1.take(), lo, hi);
        }
        // なぞのお守りは第 2 スキルが付かない
        if t.no_skill2 < 100 {
            for (&s, &(_, max)) in t.skill2.iter().zip(t.pts2) {
                let i = info.entry(s).or_insert_with(|| new(s));
                i.pts2 = union(i.pts2.take(), 1, max);
            }
        }
    }
    info.into_values().collect()
}
