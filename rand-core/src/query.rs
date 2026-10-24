//! 探すお守り。作るときに表を引き、出ないお守りなら弾く (本家の parameter)。
//!
//! 種類を決めないこともできる (鑑定画面で読んだ護石は、風化・古びたのどちらだったか分からないので)。
//! そのときは、その錬金で出る種類の表を全部引く。

use crate::charm::{Charm, Kind};
use crate::melding::{Melding, Rank};

/// 表の中の位置 (本家の p のうち、表から決まる部分)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Lookup {
    /// 第 1 スキルが表の何番目か (乱数 % 表の長さ がこれなら第 1 スキルが合う)
    pub loc1: u32,
    /// 第 1 スキルのポイントの幅 (最大 - 最小 + 1) と、探すポイント - 最小 (乱数 % 幅 がこれならポイントが合う)
    pub width1: u32,
    pub rem1: u32,
    /// 第 2 スキルが表の何番目か。None は第 2 スキルなしを探す
    pub loc2: Option<u32>,
}

/// その種類で出ない理由
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// 第 1 スキルになれない
    Skill1,
    /// 第 1 スキルのポイントが範囲の外
    Pts1,
    /// 第 2 スキルになれない (なぞのお守りは第 2 スキルが付かない)
    Skill2,
    /// 第 2 スキルのポイントが範囲の外 (1〜最大。錬金ではマイナスは出ない)
    Pts2,
    /// 第 2 スキルが第 1 スキルと同じ (画面には出ない)
    SameSkill,
    /// スロットが 0〜3 の外
    Slots,
    /// その錬金ではこの種類が出ない (マカフシギ錬金の光るお守りなど)
    NotInMelding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryError {
    /// どの種類でも出ない。種類ごとの理由
    Impossible(Vec<(Kind, Reason)>),
}

/// 探すお守り。melding と rank は、見つけた位置を錬金のフレームに直すとき
/// ([`crate::Found::aimpoints`]) に使う (お守り 1 個の作り方は変わらない)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub(crate) melding: Melding,
    pub(crate) rank: Rank,
    pub(crate) charm: Charm,
    /// 指定された種類 (None はその錬金で出る種類のどれか)
    pub(crate) kind: Option<Kind>,
    /// 探す種類と、その表の中の位置 (その種類で出うるものだけ。melding.kinds() の順)
    pub(crate) lookups: Vec<(Kind, Lookup)>,
}

impl Query {
    /// melding (rank) の錬金で出る kind の種類のお守りとして、表を引く。
    /// kind が None なら、その錬金で出る種類の表を全部引き、出うる種類だけで探す (どの種類でも出なければ Err)
    pub fn new(
        melding: Melding,
        rank: Rank,
        kind: Option<Kind>,
        charm: &Charm,
    ) -> Result<Self, QueryError> {
        let kinds: Vec<Kind> = match kind {
            Some(k) if !melding.kinds().contains(&k) => {
                return Err(QueryError::Impossible(vec![(k, Reason::NotInMelding)]));
            }
            Some(k) => vec![k],
            None => melding.kinds().to_vec(),
        };
        let found: Vec<(Kind, Result<Lookup, Reason>)> =
            kinds.into_iter().map(|k| (k, lookup(k, charm))).collect();
        if found.iter().all(|(_, r)| r.is_err()) {
            return Err(QueryError::Impossible(
                found
                    .into_iter()
                    .map(|(k, r)| (k, r.unwrap_err()))
                    .collect(),
            ));
        }
        let lookups = found
            .into_iter()
            .filter_map(|(k, r)| r.ok().map(|l| (k, l)))
            .collect();
        Ok(Self {
            melding,
            rank,
            charm: *charm,
            kind,
            lookups,
        })
    }

    /// 指定された種類 (None はその錬金で出る種類のどれか)
    pub fn kind(&self) -> Option<Kind> {
        self.kind
    }

    /// 実際に探す種類 (そのお守りが出うるもの)
    pub fn kinds(&self) -> impl Iterator<Item = Kind> + '_ {
        self.lookups.iter().map(|&(k, _)| k)
    }

    pub fn melding(&self) -> Melding {
        self.melding
    }

    pub fn rank(&self) -> Rank {
        self.rank
    }

    pub fn charm(&self) -> &Charm {
        &self.charm
    }
}

/// kind の表で charm が出うるか確かめ、表の中の位置を返す
fn lookup(kind: Kind, charm: &Charm) -> Result<Lookup, Reason> {
    let t = kind.table();
    let id1 = t
        .skill1
        .iter()
        .position(|&s| s == charm.skill1)
        .ok_or(Reason::Skill1)?;
    let (lo, hi) = t.pts1[id1];
    if !(lo as i8..=hi as i8).contains(&charm.pts1) {
        return Err(Reason::Pts1);
    }
    if charm.slots > 3 {
        return Err(Reason::Slots);
    }
    let loc2 = match charm.skill2 {
        None => None,
        Some((s2, p2)) => {
            if s2 == charm.skill1 {
                return Err(Reason::SameSkill);
            }
            // なぞのお守りは第 2 スキルが付かない (乱数 % 100 >= 100 にならない)
            let id2 = t
                .skill2
                .iter()
                .position(|&s| s == s2)
                .filter(|_| t.no_skill2 < 100)
                .ok_or(Reason::Skill2)?;
            if !(1..=t.pts2[id2].1 as i8).contains(&p2) {
                return Err(Reason::Pts2);
            }
            Some(id2 as u32)
        }
    };
    Ok(Lookup {
        loc1: id1 as u32,
        width1: u32::from(hi - lo + 1),
        rem1: (charm.pts1 - lo as i8) as u32,
        loc2,
    })
}
