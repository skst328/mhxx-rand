//! MHXX の乱数と、マカ錬金のお守りの生成・検索。apmnnn/mhxx-rng (https://github.com/apmnnn/mhxx-rng) の非公式な Rust 移植。
//!
//! 本家と違い、状態は持たない (お守りの種類や錬金のやり方は毎回渡す)。表示もしない (位置やお守りを返す)。
//! 本家のクエスト報酬 ('炭鉱') のお守りは扱わない。
//!
//! | モジュール | 中身 | 本家の対応 |
//! |---|---|---|
//! | [`rng`] | 乱数 (xorshift128) と、位置へ一気に飛ぶ jump | `ascend` `descend` `jump` |
//! | [`skill`] | お守りに付きうるスキル | スキル番号 |
//! | [`table`] | お守りの種類ごとの表 | `set_blue` など |
//! | [`charm`] | お守りの種類と、乱数からお守りを作る | `getcharm` |
//! | [`melding`] | 錬金で出るお守り | `jujupush` `halcyonpush` `around` |
//! | [`query`] | 探すお守り (表を引いた後のもの) | `parameter` |
//! | [`search`] | お守りの検索と、錬金のフレームの検索 | `search` `search_greater` |
//! | [`found`] | 検索で見つけた 1 件と、錬金のフレームへの変換 | `aimpoint_melding` |
//! | [`time`] | フレームと時間 | `watch` |
//! | [`combo`] | 調合の生産数の並びから位置を探す (ずれの診断付き) | `search_combo` |

pub mod charm;
pub mod combo;
pub mod found;
pub mod melding;
pub mod query;
pub mod rng;
pub mod search;
pub mod skill;
pub mod table;
pub mod time;

pub use charm::{Charm, Generated, Kind, charm_at, make};
pub use combo::{
    ComboError, ComboFound, ComboQuery, Diagnose, Diagnosis, Drift, diagnose_combo, search_combo,
};
pub use found::{Aimpoint, Found};
pub use melding::{Melding, Outcome, Rank, SkillInfo, meld, outcomes, skill_info};
pub use query::{Query, QueryError, Reason};
pub use search::{
    Compare, FrameMatch, MeldingError, MeldingTarget, search, search_many, search_meldings,
};
pub use skill::Skill;
pub use time::{FPS, GameTime};
