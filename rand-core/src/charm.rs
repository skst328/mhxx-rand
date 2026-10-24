//! お守りの種類と、乱数からお守りを作る (本家の getcharm の、マカ錬金の分)。
//!
//! ライセンスは [`crate::rng`] の記載を参照 (apmnnn/mhxx-rng, MIT)。

use crate::Skill;
use crate::rng::{SEED, ascend, descend, jump};
use crate::table::{MYSTERY, SHINING, Table, WEATHERED, WORN};

/// お守りの種類 (鑑定する前のお守り)。種類ごとに表が違う
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// 風化したお守り (本家の set_blue)
    Weathered,
    /// 古びたお守り (set_red)
    Worn,
    /// 光るお守り (set_yellow)
    Shining,
    /// なぞのお守り (set_white)
    Mystery,
}

impl Kind {
    pub const fn table(self) -> &'static Table {
        match self {
            Self::Weathered => &WEATHERED,
            Self::Worn => &WORN,
            Self::Shining => &SHINING,
            Self::Mystery => &MYSTERY,
        }
    }
}

/// 鑑定画面に出るお守りの中身
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Charm {
    pub skill1: Skill,
    pub pts1: i8,
    /// 第 2 スキルとポイント。None は第 2 スキルなし
    pub skill2: Option<(Skill, i8)>,
    /// 0〜3
    pub slots: u8,
}

/// 乱数から作ったお守り。鑑定画面に出ない値も持つ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Generated {
    pub skill1: Skill,
    pub pts1: i8,
    /// 第 2 スキルとポイント。第 1 スキルと同じスキルのときも入る (画面には出ない)
    pub skill2: Option<(Skill, i8)>,
    pub slots: u8,
    /// 充足値 (1〜20)
    pub fill: u8,
    /// レア度
    pub rarity: u8,
}

impl Generated {
    /// 鑑定画面に出る中身
    pub fn shown(&self) -> Charm {
        Charm {
            skill1: self.skill1,
            pts1: self.pts1,
            skill2: self.skill2.filter(|&(s, _)| s != self.skill1),
            slots: self.slots,
        }
    }
}

/// 7 個の乱数 r (r[0] が古い) から、kind の種類のお守りを作る (マカ錬金。r[6] は使わない)
pub fn make(kind: Kind, r: &[u32; 7]) -> Generated {
    let t = kind.table();
    let id1 = r[0] as usize % t.skill1.len();
    let id2 = r[3] as usize % t.skill2.len();
    let (lo1, s1) = t.pts1[id1];
    let s2 = t.pts2[id2].1;
    let pts1 = r[1] % u32::from(s1 - lo1 + 1) + u32::from(lo1);
    // (第 2 スキル, 充足値に数える第 2 スキルのポイント, スロットを決める乱数)
    let (skill2, counted2, q5) = if r[2] % 100 >= t.no_skill2 {
        let pts2 = r[4] % u32::from(s2) + 1;
        // 第 1 スキルと同じスキルなら、充足値には数えない
        let counted = if t.skill1[id1] == t.skill2[id2] {
            0
        } else {
            pts2
        };
        (Some((t.skill2[id2], pts2 as i8)), counted, r[5])
    } else {
        (None, 0, r[3])
    };
    let (s1, s2) = (u32::from(s1), u32::from(s2));
    let fill = (pts1 * s2 + counted2 * s1) * 10 / (s1 * s2);
    debug_assert!((1..=20).contains(&fill), "充足値 {fill}");
    let v = (q5 % 100) as u8;
    let (a, b, c) = t.slots[fill as usize - 1];
    let slots = if v >= c {
        3
    } else if v >= b {
        2
    } else if v >= a {
        1
    } else {
        0
    };
    Generated {
        skill1: t.skill1[id1],
        pts1: pts1 as i8,
        skill2,
        slots,
        fill: fill as u8,
        rarity: rarity(kind, slots, fill as u8),
    }
}

/// レア度 (本家の rare)
fn rarity(kind: Kind, slots: u8, fill: u8) -> u8 {
    let n = slots * 2 + fill;
    match kind {
        Kind::Weathered => 8 + u8::from(n >= 8) + u8::from(n >= 13),
        Kind::Worn => 5 + u8::from(n >= 8) + u8::from(n >= 13),
        Kind::Shining => 3 + u8::from(n >= 8),
        Kind::Mystery => 1 + u8::from(n >= 8),
    }
}

/// 位置 frame から始まる乱数で作るお守り (本家の jump(frame) してからの getcharm)
pub fn charm_at(kind: Kind, frame: u64) -> Generated {
    make(kind, &window(frame))
}

/// 位置 frame から 7 個の乱数
fn window(frame: u64) -> [u32; 7] {
    let mut s = jump(frame);
    std::array::from_fn(|_| {
        let w = s[3];
        s = ascend(s);
        w
    })
}

/// 位置 f の乱数の状態 (f = -1 は初期値の 1 つ前)
pub(crate) fn state(f: i64) -> [u32; 4] {
    if f < 0 { descend(SEED) } else { jump(f as u64) }
}
