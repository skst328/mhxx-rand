//! クエスト報酬で鑑定した護石から、「鑑定する」を押したフレームを探す。
//!
//! 報酬のお守りは、「鑑定する」を押したフレーム F の位置から 1 個目が作られ、2 個目からはその続きの乱数で作られる
//! (1 個につき、第 2 スキルが付かなければ 4、付けば 7 個の乱数を使う)。錬金と違い、何個出るかやお守りの種類は
//! 乱数では決まらない (報酬の中身で決まる) ので、鑑定した護石を 1 個目から順に渡してもらう。
//!
//! ライセンスは [`crate::rng`] の記載を参照 (apmnnn/mhxx-rng, MIT)。

use std::ops::Range;

use crate::charm::{Charm, Kind, make_reward};
use crate::rng::ascend;
use crate::search::scan;

/// クエスト報酬で鑑定した護石 1 個: レア度 (護石の名前から分かる。元のお守りの種類もこれで決まる) と中身
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewardCharm {
    /// 1〜10
    pub rarity: u8,
    /// 鑑定画面に出る中身。第 2 スキルが 0 ポイントだと画面に出ないので、なし (None) と書く
    pub charm: Charm,
}

/// search_rewards に渡したものの誤り (呼ぶ側のバグ)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RewardError {
    /// レア度が 1〜10 でない (index は charms の何番目か)
    Rarity { index: usize, rarity: u8 },
}

impl std::fmt::Display for RewardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rarity { index, rarity } => {
                write!(
                    f,
                    "レア度が 1〜10 でない (charms[{index}] のレア度 {rarity})"
                )
            }
        }
    }
}

impl std::error::Error for RewardError {}

/// frames の範囲で「鑑定する」を押すと、charms が 1 個目から順に (レア度も合わせて) 出るフレームを返す
/// (小さい順)。charms は報酬で鑑定した護石の 1 個目から何個目までか (途中から後ろは渡さなくてよい)。空なら空。
///
/// 1 個目はフレームの位置から作られるので、各フレームで 1 個目の第 1 スキルとそのポイントを手早く見比べ、
/// 通ったフレームだけ 1 個目から順にお守りを作って確かめる
pub fn search_rewards(charms: &[RewardCharm], frames: Range<u64>) -> Result<Vec<u64>, RewardError> {
    let kinds = charms
        .iter()
        .enumerate()
        .map(|(index, c)| {
            Kind::from_rarity(c.rarity).ok_or(RewardError::Rarity {
                index,
                rarity: c.rarity,
            })
        })
        .collect::<Result<Vec<Kind>, _>>()?;
    let Some(first) = charms.first() else {
        return Ok(Vec::new());
    };
    // 1 個目の第 1 スキルが表に無いか、ポイントが範囲の外なら、どこでも出ない
    let t = kinds[0].table();
    let Some(id1) = t.skill1.iter().position(|&s| s == first.charm.skill1) else {
        return Ok(Vec::new());
    };
    let (lo, hi) = t.pts1[id1];
    if !(lo as i8..=hi as i8).contains(&first.charm.pts1) {
        return Ok(Vec::new());
    }
    let (len1, id1) = (t.skill1.len() as u32, id1 as u32);
    let (width1, rem1) = (u32::from(hi - lo + 1), (first.charm.pts1 - lo as i8) as u32);
    let quick = |s: &[u32; 4]| s[0] % len1 == id1 && s[1] % width1 == rem1;
    let full = |s: &[u32; 4]| chain_matches(s, charms, &kinds);
    Ok(scan(frames, quick, full))
}

/// 位置 p から鑑定したとき、charms が順に出るか。s は位置 p〜p+3 の乱数 (位置 p+3 の状態)
fn chain_matches(s: &[u32; 4], charms: &[RewardCharm], kinds: &[Kind]) -> bool {
    // 位置 p からの乱数を、要るところまで順に作る
    let mut r: Vec<u32> = s.to_vec();
    let mut state = *s;
    let mut at = 0;
    for (c, &kind) in charms.iter().zip(kinds) {
        while r.len() < at + 7 {
            state = ascend(state);
            r.push(state[3]);
        }
        let seven: &[u32; 7] = r[at..at + 7].try_into().expect("7 個ある");
        let made = make_reward(kind, seven);
        if made.rarity != c.rarity || made.shown() != c.charm {
            return false;
        }
        at += if seven[2] % 100 >= kind.table().no_skill2 {
            7
        } else {
            4
        };
    }
    true
}
