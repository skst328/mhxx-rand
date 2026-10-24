//! Python の入口。本体 (mhxx_rand) を呼んで、Python の値に変えるだけ。
//! 重い処理は GIL を外して回す (ほかの Python のスレッドを止めない)

use std::ops::Range;

use mhxx_rand as rand;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rayon::prelude::*;

/// お守りの種類
#[pyclass(eq, eq_int, hash, frozen, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    Weathered,
    Worn,
    Shining,
    Mystery,
}

impl From<Kind> for rand::Kind {
    fn from(k: Kind) -> Self {
        match k {
            Kind::Weathered => Self::Weathered,
            Kind::Worn => Self::Worn,
            Kind::Shining => Self::Shining,
            Kind::Mystery => Self::Mystery,
        }
    }
}

impl From<rand::Kind> for Kind {
    fn from(k: rand::Kind) -> Self {
        match k {
            rand::Kind::Weathered => Self::Weathered,
            rand::Kind::Worn => Self::Worn,
            rand::Kind::Shining => Self::Shining,
            rand::Kind::Mystery => Self::Mystery,
        }
    }
}

/// 錬金のやり方
#[pyclass(eq, eq_int, hash, frozen, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Melding {
    Juju,
    Halcyon,
}

impl From<Melding> for rand::Melding {
    fn from(m: Melding) -> Self {
        match m {
            Melding::Juju => Self::Juju,
            Melding::Halcyon => Self::Halcyon,
        }
    }
}

impl From<rand::Melding> for Melding {
    fn from(m: rand::Melding) -> Self {
        match m {
            rand::Melding::Juju => Self::Juju,
            rand::Melding::Halcyon => Self::Halcyon,
        }
    }
}

/// 投入する素材のランク (1〜4)
fn rank(rank: u8) -> PyResult<rand::Rank> {
    rand::Rank::new(rank).ok_or_else(|| PyValueError::new_err(format!("rank は 1〜4: {rank}")))
}

/// 1 回の錬金の結果
#[pyclass(frozen)]
struct Outcome(rand::Outcome);

#[pymethods]
impl Outcome {
    /// 錬金したフレーム
    #[getter]
    fn frame(&self) -> u64 {
        self.0.frame
    }

    /// 出たお守り (鑑定ボックスの並び順) の種類と中身
    #[getter]
    fn charms(&self) -> Vec<(Kind, Charm)> {
        self.0
            .charms
            .iter()
            .map(|&(k, c)| (k.into(), Charm(c)))
            .collect()
    }

    /// frame までの時間
    fn game_time(&self) -> GameTime {
        GameTime(self.0.game_time())
    }

    fn __repr__(&self) -> String {
        let charms: Vec<String> = self
            .0
            .charms
            .iter()
            .map(|&(k, c)| format!("(Kind.{k:?}, {})", Charm(c).__repr__()))
            .collect();
        format!(
            "Outcome(frame={}, charms=[{}])",
            self.0.frame,
            charms.join(", ")
        )
    }
}

/// その錬金で出うるスキルと、第 1・第 2 スキルとして出るときのポイントの範囲 ((最小, 最大)。なれなければ None)
#[pyclass(frozen)]
struct SkillInfo(rand::SkillInfo);

#[pymethods]
impl SkillInfo {
    #[getter]
    fn skill(&self) -> Skill {
        Skill(self.0.skill)
    }

    #[getter]
    fn pts1(&self) -> Option<(i8, i8)> {
        self.0.pts1.as_ref().map(|r| (*r.start(), *r.end()))
    }

    #[getter]
    fn pts2(&self) -> Option<(i8, i8)> {
        self.0.pts2.as_ref().map(|r| (*r.start(), *r.end()))
    }

    fn __repr__(&self) -> String {
        let r = |p: Option<(i8, i8)>| p.map_or("None".to_string(), |(a, b)| format!("({a}, {b})"));
        format!(
            "SkillInfo(skill=Skill(\"{}\"), pts1={}, pts2={})",
            self.0.skill.name(),
            r(self.pts1()),
            r(self.pts2())
        )
    }
}

/// melding で出うるスキル (番号の小さい順) と、ポイントの範囲
#[pyfunction]
fn skill_info(melding: Melding) -> Vec<SkillInfo> {
    rand::skill_info(melding.into())
        .into_iter()
        .map(SkillInfo)
        .collect()
}

/// [start, end) の範囲のフレームごとに、melding (rank) で錬金したときの結果 (フレームの小さい順)
#[pyfunction]
#[pyo3(signature = (melding, start, end, rank = 4))]
fn outcomes(melding: Melding, start: u64, end: u64, rank: u8) -> PyResult<Vec<Outcome>> {
    Ok(
        rand::outcomes(melding.into(), self::rank(rank)?, start..end)
            .into_iter()
            .map(Outcome)
            .collect(),
    )
}

/// フレーム数で表した時間 (30 fps)。GameTime(4723) のように作る
#[pyclass(eq, ord, hash, frozen, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct GameTime(rand::GameTime);

#[pymethods]
impl GameTime {
    #[new]
    fn new(frames: u64) -> Self {
        Self(rand::GameTime(frames))
    }

    #[getter]
    fn total_frames(&self) -> u64 {
        self.0.total_frames()
    }

    #[getter]
    fn days(&self) -> u64 {
        self.0.days()
    }

    #[getter]
    fn hours(&self) -> u64 {
        self.0.hours()
    }

    #[getter]
    fn minutes(&self) -> u64 {
        self.0.minutes()
    }

    #[getter]
    fn seconds(&self) -> u64 {
        self.0.seconds()
    }

    #[getter]
    fn frames(&self) -> u64 {
        self.0.frames()
    }

    /// datetime.timedelta に (秒に満たないフレームは小数の秒になる)
    #[pyo3(name = "to_timedelta")]
    fn timedelta(&self) -> std::time::Duration {
        self.0.to_duration()
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!("GameTime({})", self.0.total_frames())
    }
}

/// 見つけた位置に届く錬金のフレームと、そのお守りが何個目か
#[pyclass(frozen, get_all)]
struct Aimpoint {
    start: u64,
    index: u8,
}

#[pymethods]
impl Aimpoint {
    /// start までの時間
    fn game_time(&self) -> GameTime {
        GameTime(rand::GameTime(self.start))
    }

    fn __repr__(&self) -> String {
        format!("Aimpoint(start={}, index={})", self.start, self.index)
    }
}

/// 検索で見つけた 1 件
#[pyclass(frozen)]
struct Found(rand::Found);

#[pymethods]
impl Found {
    /// そのお守りを作る乱数の始まり
    #[getter]
    fn position(&self) -> u64 {
        self.0.position
    }

    /// 探した Query の何番目で見つかったか (search では 0)
    #[getter]
    fn index(&self) -> usize {
        self.0.index
    }

    /// どの種類として見つかったか
    #[getter]
    fn kind(&self) -> Kind {
        self.0.kind.into()
    }

    /// 探した Query
    #[getter]
    fn query(&self) -> Query {
        Query(self.0.query().clone())
    }

    /// 鑑定画面に出るお守りの中身 (触ったときに作る)
    #[getter]
    fn charm(&self) -> Charm {
        Charm(self.0.charm())
    }

    /// この位置に届く錬金のフレームと何個目か
    fn aimpoints(&self) -> Vec<Aimpoint> {
        self.0
            .aimpoints()
            .into_iter()
            .map(|a| Aimpoint {
                start: a.start,
                index: a.index,
            })
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "Found(position={}, kind=Kind.{:?}, charm={})",
            self.0.position,
            self.0.kind,
            Charm(self.0.charm()).__repr__()
        )
    }
}

/// 見つけたお守りと、探すお守りの比べ方
#[pyclass(eq, eq_int, hash, frozen, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Compare {
    Exact,
    AtLeast,
}

impl From<Compare> for rand::Compare {
    fn from(c: Compare) -> Self {
        match c {
            Compare::Exact => Self::Exact,
            Compare::AtLeast => Self::AtLeast,
        }
    }
}

/// お守りに付きうるスキル。Skill("連撃") のように、本家の日本語の略称で作る
#[pyclass(eq, hash, frozen, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Skill(rand::Skill);

#[pymethods]
impl Skill {
    #[new]
    fn new(name: &str) -> PyResult<Self> {
        rand::Skill::from_name(name)
            .map(Self)
            .ok_or_else(|| PyValueError::new_err(format!("お守りに付かないスキル: {name}")))
    }

    /// スキル番号から
    #[staticmethod]
    fn from_id(id: u16) -> PyResult<Self> {
        rand::Skill::from_id(id)
            .map(Self)
            .ok_or_else(|| PyValueError::new_err(format!("お守りに付かないスキル番号: {id}")))
    }

    /// お守りに付きうるスキル全部 (番号の小さい順)
    #[staticmethod]
    fn all() -> Vec<Self> {
        rand::Skill::ALL.into_iter().map(Self).collect()
    }

    #[getter]
    fn id(&self) -> u16 {
        self.0.id()
    }

    #[getter]
    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn __repr__(&self) -> String {
        format!("Skill(\"{}\")", self.0.name())
    }
}

/// 鑑定画面に出るお守りの中身。第 2 スキルとそのポイントは、両方渡すか両方渡さない (第 2 スキルなし)
#[pyclass(eq, hash, frozen, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Charm(rand::Charm);

#[pymethods]
impl Charm {
    #[new]
    #[pyo3(signature = (skill1, pts1, skill2 = None, pts2 = None, slots = 0))]
    fn new(
        skill1: Skill,
        pts1: i8,
        skill2: Option<Skill>,
        pts2: Option<i8>,
        slots: u8,
    ) -> PyResult<Self> {
        let skill2 = match (skill2, pts2) {
            (Some(s), Some(p)) => Some((s.0, p)),
            (None, None) => None,
            _ => {
                return Err(PyValueError::new_err(
                    "skill2 と pts2 は両方渡すか、両方渡さない",
                ));
            }
        };
        Ok(Self(rand::Charm {
            skill1: skill1.0,
            pts1,
            skill2,
            slots,
        }))
    }

    #[getter]
    fn skill1(&self) -> Skill {
        Skill(self.0.skill1)
    }

    #[getter]
    fn pts1(&self) -> i8 {
        self.0.pts1
    }

    #[getter]
    fn skill2(&self) -> Option<Skill> {
        self.0.skill2.map(|(s, _)| Skill(s))
    }

    #[getter]
    fn pts2(&self) -> Option<i8> {
        self.0.skill2.map(|(_, p)| p)
    }

    #[getter]
    fn slots(&self) -> u8 {
        self.0.slots
    }

    fn __repr__(&self) -> String {
        let c = &self.0;
        let skill2 = c.skill2.map_or(String::new(), |(s, p)| {
            format!(", skill2=Skill(\"{}\"), pts2={p}", s.name())
        });
        format!(
            "Charm(skill1=Skill(\"{}\"), pts1={}{skill2}, slots={})",
            c.skill1.name(),
            c.pts1,
            c.slots
        )
    }
}

/// 探すお守り。作るときに表を引き、その錬金・種類で出ないお守りなら ValueError。
/// kind を省略すると、その錬金で出る種類のどれか
#[pyclass(frozen)]
struct Query(rand::Query);

#[pymethods]
impl Query {
    #[new]
    #[pyo3(signature = (melding, charm, kind = None, rank = 4))]
    fn new(melding: Melding, charm: Charm, kind: Option<Kind>, rank: u8) -> PyResult<Self> {
        rand::Query::new(
            melding.into(),
            self::rank(rank)?,
            kind.map(Into::into),
            &charm.0,
        )
        .map(Self)
        .map_err(|e| PyValueError::new_err(format!("{e:?}")))
    }

    /// 指定された種類 (None はその錬金で出る種類のどれか)
    #[getter]
    fn kind(&self) -> Option<Kind> {
        self.0.kind().map(Into::into)
    }

    #[getter]
    fn melding(&self) -> Melding {
        self.0.melding().into()
    }

    #[getter]
    fn rank(&self) -> u8 {
        self.0.rank().get()
    }

    #[getter]
    fn charm(&self) -> Charm {
        Charm(*self.0.charm())
    }

    fn __repr__(&self) -> String {
        let kind = self
            .0
            .kind()
            .map_or("None".to_string(), |k| format!("Kind.{k:?}"));
        format!(
            "Query(melding=Melding.{:?}, charm={}, kind={kind}, rank={})",
            self.0.melding(),
            Charm(*self.0.charm()).__repr__(),
            self.0.rank().get()
        )
    }
}

/// [start, end) の範囲で、query のお守りが作られる位置を探す。nthreads は使うスレッドの数
#[pyfunction]
#[pyo3(signature = (query, start, end, compare = Compare::Exact, nthreads = 1))]
fn search(
    py: Python<'_>,
    query: &Query,
    start: u64,
    end: u64,
    compare: Compare,
    nthreads: usize,
) -> PyResult<Vec<Found>> {
    let q = &query.0;
    let compare = compare.into();
    let parts = parallel(py, start..end, nthreads, |frames| {
        rand::search(q, compare, frames)
    })?;
    Ok(parts.into_iter().flatten().map(Found).collect())
}

/// [start, end) の範囲で、queries のどれかが作られる位置を探す (Found の index が queries の何番目か)。
/// 乱数の列は、お守りの種類ごとに 1 回だけ流す
#[pyfunction]
#[pyo3(signature = (queries, start, end, compare = Compare::Exact, nthreads = 1))]
fn search_many(
    py: Python<'_>,
    queries: Vec<PyRef<'_, Query>>,
    start: u64,
    end: u64,
    compare: Compare,
    nthreads: usize,
) -> PyResult<Vec<Found>> {
    let qs: Vec<rand::Query> = queries.iter().map(|q| q.0.clone()).collect();
    let compare = compare.into();
    let parts = parallel(py, start..end, nthreads, |frames| {
        rand::search_many(&qs, compare, frames)
    })?;
    Ok(parts.into_iter().flatten().map(Found).collect())
}

/// 錬金のフレームの検索で探すお守り 1 個: Query と、わかっていれば 1 回の錬金の中で何個目か
#[pyclass(frozen)]
struct MeldingTarget(rand::MeldingTarget);

#[pymethods]
impl MeldingTarget {
    #[new]
    #[pyo3(signature = (query, index = None))]
    fn new(query: &Query, index: Option<u8>) -> Self {
        Self(rand::MeldingTarget {
            query: query.0.clone(),
            index,
        })
    }

    #[getter]
    fn query(&self) -> Query {
        Query(self.0.query.clone())
    }

    #[getter]
    fn index(&self) -> Option<u8> {
        self.0.index
    }
}

/// 錬金のフレームの検索の結果の 1 件: そのフレームで錬金すると、探したお守りが全部 (それぞれ別の位置に) 出る。
/// 何個目は u32 で持つ (Vec<u8> のままだと、Python では list ではなく bytes になる)
#[pyclass(frozen, get_all)]
struct FrameMatch {
    frame: u64,
    indexes: Vec<Vec<u32>>,
    positions: Vec<u32>,
}

#[pymethods]
impl FrameMatch {
    /// frame までの時間
    fn game_time(&self) -> GameTime {
        GameTime(rand::GameTime(self.frame))
    }

    fn __repr__(&self) -> String {
        format!(
            "FrameMatch(frame={}, indexes={:?}, positions={:?})",
            self.frame, self.indexes, self.positions
        )
    }
}

/// [start, end) の範囲の錬金のフレームのうち、targets が全部 (それぞれ別の位置に) 出るもの。
/// targets の錬金のやり方・ランクが揃っていないか、何個目がその錬金の最大の数を超えたら ValueError
#[pyfunction]
#[pyo3(signature = (targets, start, end, nthreads = 1))]
fn search_meldings(
    py: Python<'_>,
    targets: Vec<PyRef<'_, MeldingTarget>>,
    start: u64,
    end: u64,
    nthreads: usize,
) -> PyResult<Vec<FrameMatch>> {
    let ts: Vec<rand::MeldingTarget> = targets.iter().map(|t| t.0.clone()).collect();
    let parts = parallel(py, start..end, nthreads, |frames| {
        rand::search_meldings(&ts, frames)
    })?;
    let mut out = Vec::new();
    for part in parts {
        let found = part.map_err(|e| PyValueError::new_err(e.to_string()))?;
        let wide = |v: Vec<u8>| v.into_iter().map(u32::from).collect();
        out.extend(found.into_iter().map(|m| FrameMatch {
            frame: m.frame,
            indexes: m.indexes.into_iter().map(wide).collect(),
            positions: wide(m.positions),
        }));
    }
    Ok(out)
}

/// 探す調合の並び。cumulative は調合の前と、調合ごとの累計の個数 (分からないところは None)。
/// 検索に使えない並びなら ValueError
#[pyclass(frozen)]
struct ComboQuery(rand::ComboQuery);

#[pymethods]
impl ComboQuery {
    #[new]
    fn new(cumulative: Vec<Option<u8>>) -> PyResult<Self> {
        rand::ComboQuery::new(&cumulative)
            .map(Self)
            .map_err(|e| PyValueError::new_err(format!("{e:?}")))
    }

    /// 照合に使う差分の個数 (範囲 N を探すときの偽陽性の期待値は N / 2^(1.5 * これ))
    #[getter]
    fn pattern_len(&self) -> usize {
        self.0.pattern_len()
    }
}

/// 乱数の進み方が想定とずれた箇所
#[pyclass(frozen, get_all)]
struct Drift {
    craft: usize,
    steps: i32,
}

#[pymethods]
impl Drift {
    fn __repr__(&self) -> String {
        format!("Drift(craft={}, steps={})", self.craft, self.steps)
    }
}

/// 普通の検索で見つからなかったときの診断の結果
#[pyclass(frozen)]
struct Diagnosis(rand::Diagnosis);

#[pymethods]
impl Diagnosis {
    #[getter]
    fn frame(&self) -> u64 {
        self.0.frame
    }

    #[getter]
    fn drifts(&self) -> Vec<Drift> {
        self.0
            .drifts
            .iter()
            .map(|d| Drift {
                craft: d.craft,
                steps: d.steps,
            })
            .collect()
    }

    #[getter]
    fn total_drift(&self) -> i32 {
        self.0.total_drift
    }

    #[getter]
    fn leading_skipped(&self) -> bool {
        self.0.leading_skipped
    }

    fn __repr__(&self) -> String {
        let d = &self.0;
        let drifts: Vec<String> = d
            .drifts
            .iter()
            .map(|x| format!("Drift(craft={}, steps={})", x.craft, x.steps))
            .collect();
        format!(
            "Diagnosis(frame={}, drifts=[{}], total_drift={}, leading_skipped={})",
            d.frame,
            drifts.join(", "),
            d.total_drift,
            if d.leading_skipped { "True" } else { "False" }
        )
    }
}

/// search_combo の結果
#[pyclass(frozen)]
struct ComboFound {
    #[pyo3(get)]
    frames: Vec<u64>,
    diagnosis: Option<rand::Diagnosis>,
}

#[pymethods]
impl ComboFound {
    /// 診断で位置が決まったときだけ
    #[getter]
    fn diagnosis(&self) -> Option<Diagnosis> {
        self.diagnosis.clone().map(Diagnosis)
    }

    fn __repr__(&self) -> String {
        let diagnosis = self
            .diagnosis()
            .map_or("None".to_string(), |d| d.__repr__());
        format!(
            "ComboFound(frames={:?}, diagnosis={diagnosis})",
            self.frames
        )
    }
}

/// [start, end) の範囲で、query の調合の並びが出るフレームを探す。diagnose なら、1 件も見つからなかったときだけ
/// 範囲全体で診断する。nthreads で並列に探すときも、診断は最後に 1 回だけ (範囲全体で投票するので区切れない)
#[pyfunction]
#[pyo3(signature = (query, start, end, diagnose = true, nthreads = 1))]
fn search_combo(
    py: Python<'_>,
    query: &ComboQuery,
    start: u64,
    end: u64,
    diagnose: bool,
    nthreads: usize,
) -> PyResult<ComboFound> {
    let q = &query.0;
    let parts = parallel(py, start..end, nthreads, |frames| {
        rand::search_combo(q, frames, rand::Diagnose::Never, None).frames
    })?;
    let frames: Vec<u64> = parts.into_iter().flatten().collect();
    let diagnosis = if diagnose && frames.is_empty() {
        py.detach(|| rand::diagnose_combo(q, start..end))
    } else {
        None
    };
    Ok(ComboFound { frames, diagnosis })
}

/// frames を nthreads 個のつながった区切りに分けて f を並列に呼び、区切りの順に結果を返す
/// (後ろの区切りほど位置が大きいので、前からつなげば小さい順)。GIL は外して回す
fn parallel<T: Send>(
    py: Python<'_>,
    frames: Range<u64>,
    nthreads: usize,
    f: impl Fn(Range<u64>) -> T + Sync,
) -> PyResult<Vec<T>> {
    if nthreads == 0 {
        return Err(PyValueError::new_err("nthreads は 1 以上"));
    }
    if nthreads == 1 {
        return Ok(vec![py.detach(|| f(frames))]);
    }
    // k 番目の区切りの始まり (n * k が u64 を超えないように u128 で計算する)
    let n = u128::from(frames.end.saturating_sub(frames.start));
    let at = |k: usize| frames.start + (n * k as u128 / nthreads as u128) as u64;
    let parts: Vec<Range<u64>> = (0..nthreads).map(|k| at(k)..at(k + 1)).collect();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(nthreads)
        .build()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(py.detach(|| pool.install(|| parts.into_par_iter().map(&f).collect())))
}

#[pymodule]
fn _mhxx_rand(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Skill>()?;
    m.add_class::<Charm>()?;
    m.add_class::<Kind>()?;
    m.add_class::<Melding>()?;
    m.add_class::<Outcome>()?;
    m.add_class::<GameTime>()?;
    m.add_class::<Aimpoint>()?;
    m.add_class::<Found>()?;
    m.add_class::<Compare>()?;
    m.add_class::<Query>()?;
    m.add_function(wrap_pyfunction!(search, m)?)?;
    m.add_function(wrap_pyfunction!(search_many, m)?)?;
    m.add_function(wrap_pyfunction!(outcomes, m)?)?;
    m.add_class::<SkillInfo>()?;
    m.add_function(wrap_pyfunction!(skill_info, m)?)?;
    m.add_class::<MeldingTarget>()?;
    m.add_class::<FrameMatch>()?;
    m.add_function(wrap_pyfunction!(search_meldings, m)?)?;
    m.add_class::<ComboQuery>()?;
    m.add_class::<Drift>()?;
    m.add_class::<Diagnosis>()?;
    m.add_class::<ComboFound>()?;
    m.add_function(wrap_pyfunction!(search_combo, m)?)?;
    Ok(())
}
