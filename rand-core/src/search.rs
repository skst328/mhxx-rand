//! お守りの検索と、錬金のフレームの検索。どれも位置の範囲 [start, end) を探し、見つけた位置を小さい順に返す。
//! 範囲で区切って呼べるので、進み具合の表示や並列化は呼ぶ側が範囲を刻めばよい。

use std::collections::BTreeSet;
use std::ops::Range;
use std::sync::Arc;

use crate::charm::{Charm, Kind, make, state};
use crate::found::Found;
use crate::melding::{kinds, meld_with};
use crate::query::{Lookup, Query};
use crate::rng::{ascend, jump};

/// 見つけたお守りと、探すお守りの比べ方
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compare {
    /// 中身が同じ (本家の search)
    Exact,
    /// スキルが同じで、ポイントとスロットが以上 (本家の search_greater)
    AtLeast,
}

/// frames の範囲で、query のお守りが作られる位置 (お守りの最初の乱数の位置) を探す。
/// AtLeast で第 2 スキルを指定しなければ、第 2 スキルは問わない (本家の search_greater で '' にしたとき)。
/// Query の種類が決まっていなければ、出うる種類ごとに探し、[`Found::kind`] にどの種類として見つかったかを入れる。
/// 見つけた位置の小さい順 (同じ位置なら [`Kind`] の定義の順) に返す。錬金のフレームへは [`Found::aimpoints`] で直す
pub fn search(query: &Query, compare: Compare, frames: Range<u64>) -> Vec<Found> {
    search_many(std::slice::from_ref(query), compare, frames)
}

/// kind の表で、lookup・want のお守りが作られる位置を探す (乱数の列を 1 回流す)
fn search_kind(
    kind: Kind,
    lookup: &Lookup,
    want: &Charm,
    compare: Compare,
    frames: Range<u64>,
) -> Vec<u64> {
    let t = kind.table();
    let l = lookup;
    let skill1 = Remainder::new(t.skill1.len() as u32, l.loc1);
    let width1 = FastMod::new(l.width1);
    let pts1 = pts1_mask(compare, l.width1, l.rem1);
    let skill2 = l.loc2.map(|loc2| {
        let pts2 = want.skill2.expect("loc2 があれば第 2 スキルがある").1;
        let mut p = Pts2::new(t.pts2[loc2 as usize].1);
        p.add(compare, pts2);
        (Remainder::new(t.skill2.len() as u32, loc2), p)
    });
    // 作る前に手早く弾く: 第 1 スキルとそのポイント、指定があれば第 2 スキルが付くかとその種類とポイント
    let quick = |s: &[u32; 4]| {
        skill1.matches(s[0])
            && pts1 >> width1.rem(s[1]) & 1 == 1
            && skill2
                .is_none_or(|(m, p)| s[2] % 100 >= t.no_skill2 && m.matches(s[3]) && p.passes(s))
    };
    let full = |s: &[u32; 4]| {
        let c = make(kind, &seven(s)).shown();
        match compare {
            Compare::Exact => c == *want,
            Compare::AtLeast => at_least(&c, want),
        }
    };
    scan(frames, quick, full)
}

/// queries のどれかが作られる位置を探す。位置の小さい順、同じ位置なら queries の順 ([`Found::index`] が何番目か)、
/// さらに同じなら [`Kind`] の定義の順。乱数の列は、お守りの種類ごとに 1 回だけ流す (本家の show_fast_multi)。
/// Query 1 個だけなら、1 回で済む専用の探し方 (search_kind) を使う
pub fn search_many(queries: &[Query], compare: Compare, frames: Range<u64>) -> Vec<Found> {
    // 種類ごとに、その種類で探す (queries の何番目か, 表の中の位置)
    let mut groups: Vec<(Kind, Vec<(usize, &Lookup)>)> = Vec::new();
    for (i, q) in queries.iter().enumerate() {
        for (k, l) in &q.lookups {
            match groups.iter_mut().find(|(g, _)| g == k) {
                Some((_, v)) => v.push((i, l)),
                None => groups.push((*k, vec![(i, l)])),
            }
        }
    }
    let qs: Vec<Arc<Query>> = queries.iter().cloned().map(Arc::new).collect();
    let mut found: Vec<Found> = groups
        .iter()
        .flat_map(|(kind, entries)| {
            let hits = match entries.as_slice() {
                [(i, l)] => search_kind(*kind, l, &queries[*i].charm, compare, frames.clone())
                    .into_iter()
                    .map(|p| (p, *i))
                    .collect(),
                _ => search_group(*kind, entries, queries, compare, frames.clone()),
            };
            let qs = &qs;
            hits.into_iter().map(move |(position, index)| Found {
                position,
                index,
                kind: *kind,
                query: Arc::clone(&qs[index]),
            })
        })
        .collect();
    if groups.len() > 1 {
        found.sort_by_key(|f| (f.position, f.index, f.kind as u8));
    }
    found
}

/// 第 1 スキルのポイントを決める乱数 % 幅 のうち、通してよいもの (ビット v が立っていれば v を通す)。
/// Exact は探すポイントの 1 つ、AtLeast はそれ以上の全部
fn pts1_mask(compare: Compare, width1: u32, rem1: u32) -> u32 {
    debug_assert!(width1 <= 32 && rem1 < width1);
    match compare {
        Compare::Exact => 1 << rem1,
        Compare::AtLeast => (u32::MAX >> (32 - width1)) & !((1 << rem1) - 1),
    }
}

/// 第 2 スキルのポイントを決める乱数 % 最大 のうち、通してよいもの (ビット v が立っていれば v を通す)。
/// ポイントは r4 % 最大 + 1
#[derive(Clone, Copy)]
struct Pts2 {
    max: FastMod,
    /// ビット v は ポイント v + 1
    mask: u32,
}

impl Pts2 {
    /// 表の最大から、まだ何も通さないもの
    fn new(max: u8) -> Self {
        debug_assert!(max <= 32);
        Self {
            max: FastMod::new(u32::from(max)),
            mask: 0,
        }
    }

    /// pts2 (1 以上) を探す Query の分を通すようにする。Exact はその 1 つ、AtLeast はそれ以上の全部
    fn add(&mut self, compare: Compare, pts2: i8) {
        let ones = |n: u32| if n >= 32 { u32::MAX } else { (1 << n) - 1 };
        let v = pts2 as u32 - 1;
        self.mask |= match compare {
            Compare::Exact => 1 << v,
            Compare::AtLeast => ones(self.max.d) & !ones(v),
        };
    }

    /// 位置 p+3 の状態 s (位置 p〜p+3 の乱数) から r4 を作って見る
    #[inline]
    fn passes(&self, s: &[u32; 4]) -> bool {
        let r4 = ascend(*s)[3];
        self.mask >> self.max.rem(r4) & 1 == 1
    }
}

/// 第 1 スキルの表の位置が同じ Query の集まり
struct Bucket {
    /// 第 1 スキルのポイントの幅と、通してよい乱数 % 幅 ([`pts1_mask`] を Query の分だけ合わせたもの)
    width1: FastMod,
    pts1: u32,
    /// 第 2 スキルを指定しない Query (queries の何番目か。queries の順)
    no2: Vec<usize>,
    /// 第 2 スキルの表の位置ごとの Query と、そのポイントの見比べ (Query が無い位置は None)
    by2: Vec<Option<(Vec<usize>, Pts2)>>,
}

/// search_many の、kind の種類で探す Query (entries は queries の何番目かと、kind の表の中の位置) をまとめて探す
fn search_group(
    kind: Kind,
    entries: &[(usize, &Lookup)],
    queries: &[Query],
    compare: Compare,
    frames: Range<u64>,
) -> Vec<(u64, usize)> {
    let t = kind.table();
    let len1 = FastMod::new(t.skill1.len() as u32);
    let len2 = FastMod::new(t.skill2.len() as u32);
    let mut buckets: Vec<Bucket> = t
        .pts1
        .iter()
        .map(|&(lo, hi)| Bucket {
            width1: FastMod::new(u32::from(hi - lo + 1)),
            pts1: 0,
            no2: Vec::new(),
            by2: vec![None; t.skill2.len()],
        })
        .collect();
    for &(i, l) in entries {
        let b = &mut buckets[l.loc1 as usize];
        b.pts1 |= pts1_mask(compare, l.width1, l.rem1);
        match (l.loc2, queries[i].charm.skill2) {
            (Some(l), Some((_, pts2))) => {
                let (v, p) = b.by2[l as usize]
                    .get_or_insert_with(|| (Vec::new(), Pts2::new(t.pts2[l as usize].1)));
                v.push(i);
                p.add(compare, pts2);
            }
            _ => b.no2.push(i),
        }
    }

    // 手早い見比べを通った位置で、お守りを 1 回作り、no2 と with2 の Query と比べる (ループの中に展開させない)
    #[inline(never)]
    fn check(
        s: &[u32; 4],
        p: u64,
        no2: &[usize],
        with2: &[usize],
        make_charm: impl Fn(&[u32; 4]) -> Charm,
        test: impl Fn(&Charm, usize) -> bool,
        found: &mut Vec<(u64, usize)>,
    ) {
        let c = make_charm(s);
        let start = found.len();
        found.extend(
            no2.iter()
                .chain(with2)
                .filter(|&&i| test(&c, i))
                .map(|&i| (p, i)),
        );
        // 同じ位置は queries の順にする (no2 と with2 はそれぞれ queries の順)
        found[start..].sort_unstable();
    }
    let make_charm = |s: &[u32; 4]| make(kind, &seven(s)).shown();
    let test = |c: &Charm, i: usize| {
        let want = &queries[i].charm;
        match compare {
            Compare::Exact => c == want,
            Compare::AtLeast => at_least(c, want),
        }
    };

    let mut found = Vec::new();
    if frames.is_empty() {
        return found;
    }
    let mut s = jump(frames.start + 3);
    for p in frames {
        let b = &buckets[len1.rem(s[0]) as usize];
        if b.pts1 >> b.width1.rem(s[1]) & 1 == 1 {
            // 第 2 スキルが付けば、その種類とポイントで見比べる Query
            let with2: &[usize] =
                match (s[2] % 100 >= t.no_skill2).then(|| &b.by2[len2.rem(s[3]) as usize]) {
                    Some(Some((v, p2))) if p2.passes(&s) => v,
                    _ => &[],
                };
            if !b.no2.is_empty() || !with2.is_empty() {
                check(&s, p, &b.no2, with2, make_charm, test, &mut found);
            }
        }
        s = ascend(s);
    }
    found
}

/// a % d を、割り算の代わりに掛け算で求める (Lemire らの fastmod。a と d は 32 bit)。
/// d = 1 のときは m が 0 に回り込み、いつも 0 を返す (それで正しい)
#[derive(Clone, Copy)]
struct FastMod {
    m: u64,
    d: u32,
}

impl FastMod {
    fn new(d: u32) -> Self {
        Self {
            m: (u64::MAX / u64::from(d)).wrapping_add(1),
            d,
        }
    }

    #[inline]
    fn rem(self, a: u32) -> u32 {
        let low = self.m.wrapping_mul(u64::from(a));
        ((u128::from(low) * u128::from(self.d)) >> 64) as u32
    }
}

/// 位置 p+3 の状態 s から、位置 p〜p+6 の 7 個の乱数 (位置 p から作るお守りが使うもの) を作る。
/// xorshift128 の状態 [x, y, z, w] は直前 4 個の乱数そのものなので、s が位置 p〜p+3 の乱数になる
#[inline]
fn seven(s: &[u32; 4]) -> [u32; 7] {
    let a = ascend(*s);
    let b = ascend(a);
    let c = ascend(b);
    [s[0], s[1], s[2], s[3], a[3], b[3], c[3]]
}

/// frames の位置 p のうち、quick と full を両方通るものを小さい順に返す。どちらにも位置 p+3 の状態を渡す
/// (s[0..4] が位置 p〜p+3 の乱数。残りが要れば [`seven`] で作る)。quick は毎回呼ぶ手早い見比べで、
/// full は quick を通ったときだけ呼ぶ重い処理 (ループの中に展開させない)
fn scan(
    frames: Range<u64>,
    quick: impl Fn(&[u32; 4]) -> bool,
    full: impl Fn(&[u32; 4]) -> bool,
) -> Vec<u64> {
    #[inline(never)]
    fn slow(full: &impl Fn(&[u32; 4]) -> bool, s: &[u32; 4], p: u64, found: &mut Vec<u64>) {
        if full(s) {
            found.push(p);
        }
    }
    let mut found = Vec::new();
    if frames.is_empty() {
        return found;
    }
    let mut s = jump(frames.start + 3);
    for p in frames {
        if quick(&s) {
            slow(&full, &s, p, &mut found);
        }
        s = ascend(s);
    }
    found
}

/// 「a % d == rem か」を、割り算の代わりに掛け算で見る (本家の div_nj と同じ考え方)。
/// c = ceil(2^64 / d) とすると、a < 2^32 で「(a * c) mod 2^64 < c」と「d が a を割り切る」が同じになる
/// (Lemire らの割り切れるかの判定)。a - rem で見るので、a < rem のときを別に弾く
#[derive(Clone, Copy)]
struct Remainder {
    c: u64,
    rem: u32,
}

impl Remainder {
    fn new(d: u32, rem: u32) -> Self {
        debug_assert!(d > 1 && rem < d);
        Self {
            c: u64::MAX / u64::from(d) + 1,
            rem,
        }
    }

    #[inline]
    fn matches(self, a: u32) -> bool {
        a >= self.rem && u64::from(a - self.rem).wrapping_mul(self.c) < self.c
    }
}

/// c が want と同じスキルで、ポイントとスロットが以上か (want に第 2 スキルが無ければ、第 2 スキルは問わない)
fn at_least(c: &Charm, want: &Charm) -> bool {
    c.skill1 == want.skill1
        && c.pts1 >= want.pts1
        && c.slots >= want.slots
        && want
            .skill2
            .is_none_or(|(s, p)| c.skill2.is_some_and(|(cs, cp)| cs == s && cp >= p))
}

/// 錬金のフレームの検索で探すお守り 1 個: 中身と、わかっていれば 1 回の錬金の中で何個目か
/// (鑑定画面で読んだ護石は種類が分からないので、query は種類を決めないのが普通)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeldingTarget {
    pub query: Query,
    /// 何個目か (1 から)。None は指定なし
    pub index: Option<u8>,
}

/// 錬金のフレームの検索の結果の 1 件: そのフレームで錬金すると、探したお守りが全部 (それぞれ別の位置に) 出る
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameMatch {
    pub frame: u64,
    /// 探したお守りごと (渡した順) の、何個目に出るか (1 から、小さい順)
    pub indexes: Vec<Vec<u8>>,
    /// お守りごとの位置の選び方の 1 つ (全部が別の位置になるもの。渡した順)
    pub positions: Vec<u8>,
}

/// search_meldings に渡したものの誤り (呼ぶ側のバグ)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeldingError {
    /// targets の錬金のやり方かランクが揃っていない
    Mixed,
    /// 何個目が 1〜その錬金の最大の数でない (鑑定ボックスで読んだ位置が最大の数を超えたら、1 回の錬金の中での
    /// 位置ではないので、呼ぶ側で None にして渡す)
    Index { index: u8 },
}

impl std::fmt::Display for MeldingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mixed => write!(f, "錬金のやり方かランクが揃っていない"),
            Self::Index { index } => write!(f, "何個目が 1 回の錬金の中での位置でない ({index})"),
        }
    }
}

impl std::error::Error for MeldingError {}

/// 手元に持つ乱数の数: 錬金の 1 つ前 + 最大の間 (天運の 30) + 最後のお守りの 7 個より大きい 2 のべき
const WINDOW: usize = 64;

/// frames の範囲の錬金のフレームのうち、targets が全部 (index があればその何個目に、全部が別の位置に) 出るものを、
/// お守りごとの何個目と一緒に返す (フレームの小さい順。mhxx-renkin-reader の search_frames)。targets が空なら空。
/// targets は同じ錬金 (やり方とランク) のもの。
///
/// フレームごとに錬金をたどると重いので、逆からたどる: 乱数の列を 1 回だけ流し、「この位置の乱数から作るお守りが
/// 手がかりのお守りになる」位置 p を探す (第 1 スキルが合うだけで 1/60 程度なので、まれ)。見つかったら、p を
/// 通りうる錬金のフレーム (p の 0〜max_offset 個手前) を挙げる。挙げたフレームは、錬金の乱数がそろったところで
/// お守りを作り直し、全部のお守りが出るかを確かめる。手がかりには、いちばん出にくそうなお守りを使う
pub fn search_meldings(
    targets: &[MeldingTarget],
    frames: Range<u64>,
) -> Result<Vec<FrameMatch>, MeldingError> {
    let Some(first) = targets.first() else {
        return Ok(Vec::new());
    };
    let (melding, rank) = (first.query.melding, first.query.rank);
    if targets
        .iter()
        .any(|t| t.query.melding != melding || t.query.rank != rank)
    {
        return Err(MeldingError::Mixed);
    }
    if let Some(index) = targets
        .iter()
        .filter_map(|t| t.index)
        .find(|i| !(1..=melding.max_charms()).contains(i))
    {
        return Err(MeldingError::Index { index });
    }
    if frames.is_empty() {
        return Ok(Vec::new());
    }
    let max_offset = melding.max_offset() as usize;
    let lead = targets
        .iter()
        .min_by(|a, b| density(&a.query).total_cmp(&density(&b.query)))
        .expect("targets は空でない");
    let (want, index) = (&lead.query.charm, lead.index);
    // 手がかりの見比べに使う、種類ごとの値
    let quick: Vec<(Kind, &Lookup, u32, u32)> = lead
        .query
        .lookups
        .iter()
        .map(|(k, l)| {
            (
                *k,
                l,
                k.table().skill1.len() as u32,
                k.table().skill2.len() as u32,
            )
        })
        .collect();

    // 位置は u = フレーム - (start - 1) で数える。v[u % WINDOW] がその位置の乱数。
    // 錬金したフレーム F (u = 1 ..= n) のお守りは u = F .. F + max_offset から始まり、その乱数は 7 個先まで使う
    let n = (frames.end - frames.start) as usize;
    let last_p = n + max_offset;
    let mut v = [0u32; WINDOW];
    let mut s = state(frames.start as i64 - 1);
    // 手がかりのお守りが出うる錬金のフレーム (u)。作り直すのに要る乱数 (u - 1 .. u + max_offset + 6) がそろったら
    // 確かめる。p から挙げるフレームは p - max_offset 以上なので、確かめ終えたフレームより前のものが後から来ることはない
    let mut pending = BTreeSet::new();
    let mut found = Vec::new();
    for q in 0..=last_p + 6 {
        v[q % WINDOW] = s[3];
        s = ascend(s);
        let at = |u: usize| v[u % WINDOW];
        // 乱数が 7 個そろった位置 p から作るお守りを見る
        if let Some(p) = q.checked_sub(6).filter(|&p| p >= 1) {
            for &(k, l, len1, len2) in &quick {
                let t = k.table();
                if at(p) % len1 != l.loc1
                    || at(p + 1) % l.width1 != l.rem1
                    || l.loc2
                        .is_some_and(|l2| at(p + 2) % 100 < t.no_skill2 || at(p + 3) % len2 != l2)
                {
                    continue;
                }
                let r: [u32; 7] = std::array::from_fn(|d| at(p + d));
                if make(k, &r).shown() != *want {
                    continue;
                }
                // p を k の種類のお守りとして通る錬金のフレームを探す
                for f in p.saturating_sub(max_offset).max(1)..=p.min(n) {
                    let mut pos = f;
                    for (j, kind) in kinds(melding, rank, at(f - 1)).into_iter().enumerate() {
                        if pos >= p {
                            if pos == p && kind == k && index.is_none_or(|i| j + 1 == i as usize) {
                                pending.insert(f);
                            }
                            break;
                        }
                        pos += if at(pos + 2) % 100 < kind.table().no_skill2 {
                            4
                        } else {
                            6
                        };
                    }
                }
            }
        }
        while let Some(&u) = pending.first() {
            if u + max_offset + 6 > q {
                break;
            }
            pending.pop_first();
            let made = meld_with(melding, rank, |d| at(u - 1 + d));
            // お守りごとに、作ったお守りのうち何個目に出るか
            let indexes: Vec<Vec<u8>> = targets
                .iter()
                .map(|t| {
                    (0..made.len())
                        .filter(|&j| {
                            let (kind, c) = &made[j];
                            t.index.is_none_or(|i| j + 1 == i as usize)
                                && t.query.lookups.iter().any(|(k, _)| k == kind)
                                && c.shown() == t.query.charm
                        })
                        .map(|j| j as u8 + 1)
                        .collect()
                })
                .collect();
            if let Some(positions) = distinct_positions(&indexes) {
                found.push(FrameMatch {
                    frame: frames.start + (u - 1) as u64,
                    indexes,
                    positions,
                });
            }
        }
    }
    Ok(found)
}

/// お守りが乱数の列のある位置から作られる見込み (表からの大まかな値。スロットは見ない)。
/// 小さいほど、乱数の列を流したときに見つかる位置が少ない
fn density(q: &Query) -> f64 {
    q.lookups
        .iter()
        .map(|(k, l)| {
            let t = k.table();
            let skill2 = l.loc2.map_or(1.0, |_| {
                f64::from(100 - t.no_skill2) / 100.0 / t.skill2.len() as f64
            });
            skill2 / t.skill1.len() as f64 / f64::from(l.width1)
        })
        .sum()
}

/// お守りごとの位置の候補 (options[k] は k 個目のお守りが出る何個目) から、全部が別の位置になる選び方を 1 つ返す。
/// お守りも位置も 6 つまでなので、全部試しても軽い
fn distinct_positions(options: &[Vec<u8>]) -> Option<Vec<u8>> {
    fn pick(options: &[Vec<u8>], chosen: &mut Vec<u8>) -> bool {
        let Some((first, rest)) = options.split_first() else {
            return true;
        };
        for &p in first {
            if chosen.contains(&p) {
                continue;
            }
            chosen.push(p);
            if pick(rest, chosen) {
                return true;
            }
            chosen.pop();
        }
        false
    }
    let mut chosen = Vec::new();
    pick(options, &mut chosen).then_some(chosen)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 確かめる a: 小さい値・32 bit の上の端・その間を飛び飛びに
    fn samples() -> impl Iterator<Item = u32> {
        (0..5000)
            .chain(u32::MAX - 5000..=u32::MAX)
            .chain((0..=u32::MAX).step_by(65_521))
    }

    /// 掛け算での余りの判定 (Remainder) が、割り算と同じ。割る数は表の長さ・ポイントの幅になりうる 2〜128
    #[test]
    fn remainder_matches_division() {
        for d in 2..=128u32 {
            for rem in [0, 1, d / 2, d - 1] {
                let r = Remainder::new(d, rem);
                for a in samples() {
                    assert_eq!(r.matches(a), a % d == rem, "a {a} d {d} rem {rem}");
                }
            }
        }
    }

    /// 掛け算での余り (FastMod) が、割り算と同じ (1 で割るときも)
    #[test]
    fn fast_mod_matches_division() {
        for d in 1..=128u32 {
            let m = FastMod::new(d);
            for a in samples() {
                assert_eq!(m.rem(a), a % d, "a {a} d {d}");
            }
        }
    }
}
