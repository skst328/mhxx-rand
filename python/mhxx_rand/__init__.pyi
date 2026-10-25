"""MHXX の乱数と、マカ錬金のお守りの生成・検索。

`apmnnn/mhxx-rng <https://github.com/apmnnn/mhxx-rng>`_ の非公式な Rust 移植。
本家と違い、状態は持たず (お守りの種類や錬金のやり方は毎回渡す)、
表示もしない (位置やお守りを返す)。
本家のクエスト報酬 (``'炭鉱'``) のお守りは扱わない。

位置は、ゲームの乱数を初期値から何回進めたか
(本家の「乱数位置」・「フレーム」) で数える。
"""

from collections.abc import Sequence
from datetime import timedelta
from typing import Literal, TypeAlias, final

SkillName: TypeAlias = Literal[
    "毒", "麻痺", "睡眠", "気絶", "聴覚", "風圧", "耐震", "だる", "耐暑",
    "耐寒", "寒冷", "炎熱", "盗み", "対防", "狂撃", "細菌", "裂傷", "攻撃",
    "防御", "体力", "火耐", "水耐", "雷耐", "氷耐", "龍耐", "属耐", "火攻",
    "水攻", "雷攻", "氷攻", "龍攻", "属攻", "特攻", "研師", "匠", "斬味",
    "剣術", "研磨", "鈍器", "抜会", "抜減", "納刀", "納研", "刃鱗", "装速",
    "反動", "精密", "通強", "貫強", "散強", "重強", "通追", "貫追", "散追",
    "榴追", "拡追", "毒追", "麻追", "睡追", "強追", "属追", "接追", "減追",
    "爆追", "速射", "射法", "装数", "変則", "弾節", "達人", "痛撃", "連撃",
    "特会", "属会", "会心", "裏会", "溜短", "スタ", "体術", "気力", "走行",
    "回性", "回距", "泡沫", "ガ性", "ガ強", "KO", "減攻", "笛", "砲術", "重撃",
    "爆弾", "本気", "闘魂", "無傷", "チャ", "龍気", "底力", "逆境", "逆上",
    "窮地", "根性", "気配", "采配", "号令", "乗り", "跳躍", "無心", "我慢",
    "SP", "千里", "観察", "狩人", "運搬", "加護", "英雄", "回量", "回速",
    "効果", "広域", "腹減", "食い", "食事", "節食", "肉食", "茸食", "野草",
    "調成", "調数", "高速", "採取", "ハチ", "護石", "気ま", "運気", "剥取",
    "捕獲", "紅兜", "大雪", "矛砕", "岩穿", "紫毒", "宝纏", "白疾", "隻眼",
    "黒炎", "金雷", "荒鉤", "燼滅", "朧隠", "鎧裂", "天眼", "青電", "銀嶺",
    "鏖魔", "真紅", "真大", "真矛", "真岩", "真紫", "真宝", "真白", "真隻",
    "真黒", "真金", "真荒", "真燼", "真朧", "真鎧", "真天", "真青", "真銀",
    "真鏖",
]
"""スキルの名前 (本家の日本語の略称)。お守りに付きうる 173 個。
本家が表示の幅を揃えるために使う全角 (「毒　」の全角スペース、
「ＫＯ」「ＳＰ」) は、取るか半角にしてある (「毒」「KO」「SP」)"""

@final
class Skill:
    """お守りに付きうるスキル。
    ``Skill("連撃")`` のように、本家の日本語の略称で作る。

    :param name: スキルの名前 (:data:`SkillName` のどれか)
    :raises ValueError: お守りに付かないスキル
    """

    def __init__(self, name: SkillName) -> None: ...
    @staticmethod
    def from_id(id: int) -> Skill:
        """スキル番号 (本家と同じ番号) から作る。

        :raises ValueError: お守りに付かないスキル番号
        """
    @staticmethod
    def all() -> list[Skill]:
        """お守りに付きうるスキル全部 (番号の小さい順)"""
    @property
    def id(self) -> int:
        """スキル番号 (本家と同じ番号)"""
    @property
    def name(self) -> SkillName:
        """名前 (本家の日本語の略称)"""

@final
class Kind:
    """お守りの種類 (鑑定する前のお守り)。
    種類ごとに、出るスキルやポイントの表が違う。
    ``str()`` は鑑定画面の名前 (``風化したお守り`` など)。"""

    Weathered: Kind
    """風化したお守り (本家の ``set_blue``)"""
    Worn: Kind
    """古びたお守り (``set_red``)"""
    Shining: Kind
    """光るお守り (``set_yellow``)"""
    Mystery: Kind
    """なぞのお守り (``set_white``)。第 2 スキルは付かない"""

@final
class Melding:
    """錬金のやり方。
    1 回の錬金で出るお守りの種類と個数の決まり方が違う。"""

    Juju: Melding
    """マカフシギ錬金。風化したお守りと古びたお守りが出る
    (本家の ``jujupush``)"""
    Halcyon: Melding
    """天運の錬金。古びた・光る・なぞのお守りが出る
    (本家の ``halcyonpush``)"""

@final
class Compare:
    """見つけたお守りと、探すお守りの比べ方。"""

    Exact: Compare
    """中身が同じ (本家の ``search``)"""
    AtLeast: Compare
    """スキルが同じで、ポイントとスロットが以上
    (本家の ``search_greater``)。
    探すお守りに第 2 スキルが無ければ、第 2 スキルは問わない。"""

@final
class Charm:
    """鑑定画面に出るお守りの中身。種類や錬金のやり方には縛られない
    (同じ Charm から、種類ごとに :class:`Query` を作れる)。

    第 1 スキルと同じ第 2 スキルは画面に出ないので、
    第 2 スキルなしのお守りとして扱う。

    ``str()`` は本家の表示と同じ 1 行の形 (``連撃  4 痛撃   5 S3``。
    第 2 スキルなしは ``--------``)。``repr()`` はこのクラスを作る式の形。

    :param skill1: 第 1 スキル
    :param pts1: 第 1 スキルのポイント
    :param skill2: 第 2 スキル。None は第 2 スキルなし
    :param pts2: 第 2 スキルのポイント
        (skill2 と両方渡すか、両方渡さない)
    :param slots: スロットの数 (0〜3)
    :raises ValueError: skill2 と pts2 の片方だけを渡した
    """

    def __init__(
        self,
        skill1: Skill,
        pts1: int,
        skill2: Skill | None = None,
        pts2: int | None = None,
        slots: int = 0,
    ) -> None: ...
    @property
    def skill1(self) -> Skill: ...
    @property
    def pts1(self) -> int: ...
    @property
    def skill2(self) -> Skill | None: ...
    @property
    def pts2(self) -> int | None: ...
    @property
    def slots(self) -> int: ...

@final
class Query:
    """探すお守り。作るときに種類の表を引き、
    その錬金・種類で出ないお守りなら :class:`ValueError` にする。

    kind を省略すると、その錬金で出る種類のどれか
    (マカフシギ錬金なら風化・古びた) として探す。
    鑑定画面で読んだ護石のように、種類が分からないときに使う。
    どの種類として見つかったかは :attr:`Found.kind`。

    melding と rank は、見つけた位置を錬金のフレームに直すとき
    (:meth:`Found.aimpoints`) に使う (お守り 1 個の作り方は変わらない)。

    :param melding: 錬金のやり方
    :param charm: 探すお守りの中身
    :param kind: お守りの種類。None (省略) はその錬金で出る種類のどれか
    :param rank: 投入する素材のランク (1〜4。既定は最高ランク)
    :raises ValueError: その錬金・種類で出ないお守り
        (種類ごとの理由も付く)、rank が 1〜4 でない
    """

    def __init__(
        self,
        melding: Melding,
        charm: Charm,
        kind: Kind | None = None,
        rank: int = 4,
    ) -> None: ...
    @property
    def kind(self) -> Kind | None:
        """指定された種類 (None はその錬金で出る種類のどれか)"""
    @property
    def melding(self) -> Melding: ...
    @property
    def rank(self) -> int: ...
    @property
    def charm(self) -> Charm: ...

@final
class GameTime:
    """フレーム数で表した時間 (1 秒 30 フレーム)。
    ``str()`` は本家の ``watch`` と同じ ``0d 0h 2m 37s 13f`` の形。

    乱数は 1 フレームに 1 つ進むので、乱数の位置は、
    ゲームを始めてからの時間でもある。

    :param frames: フレーム数
    """

    def __init__(self, frames: int) -> None: ...
    @property
    def total_frames(self) -> int:
        """全体のフレーム数"""
    @property
    def days(self) -> int: ...
    @property
    def hours(self) -> int:
        """日に満たない時間 (0〜23)"""
    @property
    def minutes(self) -> int:
        """時間に満たない分 (0〜59)"""
    @property
    def seconds(self) -> int:
        """分に満たない秒 (0〜59)"""
    @property
    def frames(self) -> int:
        """秒に満たないフレーム (0〜29)"""
    def to_timedelta(self) -> timedelta:
        """:class:`datetime.timedelta` に直す
        (秒に満たないフレームは小数の秒になる)"""
    def __lt__(self, other: GameTime) -> bool: ...
    def __le__(self, other: GameTime) -> bool: ...
    def __gt__(self, other: GameTime) -> bool: ...
    def __ge__(self, other: GameTime) -> bool: ...

@final
class Aimpoint:
    """見つけた位置に届く錬金のフレームと、そのお守りが何個目か。"""

    @property
    def start(self) -> int:
        """錬金したフレーム"""
    @property
    def index(self) -> int:
        """何個目か (1 から)"""
    def game_time(self) -> GameTime:
        """start までの時間 (ゲームを始めてから錬金するまで)"""

@final
class Found:
    """:func:`search` / :func:`search_many` で見つけた 1 件。"""

    @property
    def position(self) -> int:
        """そのお守りを作る乱数の始まり"""
    @property
    def index(self) -> int:
        """:func:`search_many` に渡した queries の何番目で見つかったか
        (:func:`search` では 0)"""
    @property
    def kind(self) -> Kind:
        """どの種類として見つかったか
        (Query の種類を省略したときは、出うる種類のどれか)"""
    @property
    def query(self) -> Query:
        """探した Query"""
    @property
    def charm(self) -> Charm:
        """鑑定画面に出るお守りの中身 (触ったときに作る)。

        :attr:`Compare.AtLeast` で探したときは、
        ポイントやスロットが探したものより大きいことがある。
        """

    def aimpoints(self) -> list[Aimpoint]:
        """この位置に、:attr:`kind` の種類のお守りとして届く
        錬金のフレームと何個目かを挙げる
        (start の小さい順。本家の ``aimpoint_melding``)。

        錬金のやり方とランクは、Query のもの。
        錬金のフレームは見つけた位置より最大 18 (天運の錬金は 30)
        手前になるので、探した範囲の手前に出ることがある。
        """

def search(
    query: Query,
    start: int,
    end: int,
    compare: Compare = Compare.Exact,
    nthreads: int = 1,
) -> list[Found]:
    """位置 ``[start, end)`` の範囲で、query のお守りが作られる位置を探す
    (本家の ``search`` / ``show_fast``)。
    見つけた位置の小さい順
    (同じ位置なら風化・古びた・光る・なぞの順) に返す。
    Query の種類を省略したときは、出うる種類ごとに探す。

    :param query: 探すお守り
    :param start: 範囲の始まり
    :param end: 範囲の終わり (含まない)
    :param compare: 比べ方 (既定は :attr:`Compare.Exact`)
    :param nthreads: 使うスレッドの数。
        範囲を分けて並列に探す (結果は同じ)
    :raises ValueError: nthreads が 0
    """

def search_many(
    queries: Sequence[Query],
    start: int,
    end: int,
    compare: Compare = Compare.Exact,
    nthreads: int = 1,
) -> list[Found]:
    """位置 ``[start, end)`` の範囲で、queries のどれかが作られる位置を探す
    (本家の ``show_fast_multi``)。
    見つけた位置の小さい順、同じ位置なら queries の順、
    さらに同じなら風化・古びた・光る・なぞの順に返す
    (:attr:`Found.index` が queries の何番目か)。

    乱数の列はお守りの種類ごとに 1 回だけ流すので、queries が何千個あっても
    :func:`search` 1 回と同じくらいの速さで済む。

    :param queries: 探すお守りの一覧
        (種類や錬金のやり方が混ざっていてもよい)
    :param start: 範囲の始まり
    :param end: 範囲の終わり (含まない)
    :param compare: 比べ方 (既定は :attr:`Compare.Exact`)
    :param nthreads: 使うスレッドの数
    :raises ValueError: nthreads が 0
    """

@final
class MeldingTarget:
    """:func:`search_meldings` で探すお守り 1 個。

    鑑定画面で読んだ護石は種類が分からないので、
    query は種類を省略するのが普通。

    :param query: 探すお守り
    :param index: 1 回の錬金の中で何個目か (1 から)。None は指定なし。
        鑑定ボックスで読んだ位置がその錬金の最大の数を超えたら、
        1 回の錬金の中での位置ではないので None にする
    """

    def __init__(self, query: Query, index: int | None = None) -> None: ...
    @property
    def query(self) -> Query: ...
    @property
    def index(self) -> int | None: ...

@final
class FrameMatch:
    """:func:`search_meldings` の結果の 1 件。そのフレームで錬金すると、
    探したお守りが全部 (それぞれ別の位置に) 出る。"""

    @property
    def frame(self) -> int:
        """錬金したフレーム"""
    @property
    def indexes(self) -> list[list[int]]:
        """探したお守りごと (渡した順) の、何個目に出るか (1 から、小さい順)。
        同じ中身が 2 か所に出ることもあるので一覧"""
    @property
    def positions(self) -> list[int]:
        """お守りごとの位置の選び方の 1 つ
        (全部が別の位置になるもの。渡した順)"""
    def game_time(self) -> GameTime:
        """frame までの時間"""

def search_meldings(
    targets: Sequence[MeldingTarget],
    start: int,
    end: int,
    nthreads: int = 1,
) -> list[FrameMatch]:
    """フレーム ``[start, end)`` の範囲の錬金のフレームのうち、
    targets が全部 (それぞれ別の位置に) 出るものを探す (フレームの小さい順)。
    鑑定画面で読んだ何個もの護石から、錬金したフレームを逆算するのに使う。

    乱数の列は、いちばん出にくそうなお守りを手がかりに 1 回だけ流すので、
    お守りが何個でも速さはほぼ同じ。

    :param targets: 探すお守りの一覧 (同じ錬金のやり方・ランクのもの)
    :param start: 範囲の始まり
    :param end: 範囲の終わり (含まない)
    :param nthreads: 使うスレッドの数
    :raises ValueError: 錬金のやり方かランクが揃っていない、
        何個目がその錬金の最大の数を超えた、nthreads が 0
    """

@final
class SkillInfo:
    """その錬金で出うるスキルと、ポイントの範囲。
    画面でスキルとポイントの選択肢を作るのに使う。"""

    @property
    def skill(self) -> Skill: ...
    @property
    def pts1(self) -> tuple[int, int] | None:
        """そのスキルが第 1 スキルとして出るときの、ポイントの (最小, 最大)。
        なれなければ None"""
    @property
    def pts2(self) -> tuple[int, int] | None:
        """そのスキルが第 2 スキルとして出るときの、ポイントの (最小, 最大)。
        なれなければ None (錬金ではマイナスは出ないので 1 から)"""

def skill_info(melding: Melding) -> list[SkillInfo]:
    """melding で出うるスキル (番号の小さい順) と、ポイントの範囲。
    その錬金で出る種類の表を合わせたもの (マカフシギ錬金なら風化・古びた)。

    :param melding: 錬金のやり方
    """

@final
class ComboQuery:
    """探す調合の並び。作るときに累計の並びを差分の並びに直し、
    検索に使えなければ :class:`ValueError` にする。

    先頭 3 つは調合を始める前の乱数の消費と噛み合わないので使わず、
    上限 (99) に張り付いた最後の 1 回も使わない
    (本家の ``search_combo`` と同じ)。

    :param cumulative: 調合の前と、調合ごとの累計の個数。
        分からないところは None
    :raises ValueError: 調合が少なすぎる、累計が分からない、
        差が 2〜4 でない (位置も付く)
    """

    def __init__(self, cumulative: Sequence[int | None]) -> None: ...
    @property
    def pattern_len(self) -> int:
        """照合に使う差分の個数。範囲 N を探すときの偽陽性の期待値は
        N / 2^(1.5 * これ) (探してよい範囲の目安)"""

@final
class Drift:
    """乱数の進み方が想定とずれた箇所。"""

    @property
    def craft(self) -> int:
        """何回目の調合の直前でずれたか (1 から)"""
    @property
    def steps(self) -> int:
        """想定より何ステップ多く進んだか。少なければ負"""

@final
class Diagnosis:
    """普通の検索で見つからなかったときの診断の結果。"""

    @property
    def frame(self) -> int:
        """普通の検索が返すものと同じ基準のフレーム"""
    @property
    def drifts(self) -> list[Drift]:
        """ずれが起きた箇所"""
    @property
    def total_drift(self) -> int:
        """ずれの合計。報告値はこのぶんだけ補正が要る。
        打ち消し合っていれば 0"""
    @property
    def leading_skipped(self) -> bool:
        """先頭の 1 回を照合から外した
        (調合を始めた直後に捨てる回数が足りなかったときに起きる)"""

@final
class ComboFound:
    """:func:`search_combo` の結果。"""

    @property
    def frames(self) -> list[int]:
        """見つけたフレーム (小さい順)。本家の ``search_combo`` と同じく、
        調合を始めた位置に、調合による乱数の進み (調合回数 x 2) を
        足したもの"""
    @property
    def diagnosis(self) -> Diagnosis | None:
        """診断で位置が決まったときだけ"""

def search_combo(
    query: ComboQuery,
    start: int,
    end: int,
    diagnose: bool = True,
    nthreads: int = 1,
) -> ComboFound:
    """フレーム ``[start, end)`` の範囲で、query の調合の並びが出る
    フレームを探す (本家の ``search_combo``)。

    diagnose なら、1 件も見つからなかったときだけ、同じ範囲で途中の
    乱数のずれを診断する (調合スキャンで足したもの)。
    診断は範囲全体で投票するので、nthreads で並列に探すときも、
    最後に 1 回だけする。

    :param query: 探す調合の並び
    :param start: 範囲の始まり
    :param end: 範囲の終わり (含まない)
    :param diagnose: 見つからなかったときに診断するか
    :param nthreads: 使うスレッドの数 (普通の検索だけ並列にする)
    :raises ValueError: nthreads が 0
    """

@final
class Outcome:
    """1 回の錬金の結果。

    ``str()`` は本家の表示と同じく、出たお守りを 1 個 1 行で
    (``風化したお守り  回量  5 KO　   5 S0`` など)。
    """

    @property
    def frame(self) -> int:
        """錬金したフレーム"""
    @property
    def charms(self) -> list[tuple[Kind, Charm]]:
        """出たお守り (鑑定ボックスの並び順) の種類と、鑑定画面に出る中身"""
    def game_time(self) -> GameTime:
        """frame までの時間 (ゲームを始めてから錬金するまで)"""

def outcomes(
    melding: Melding,
    start: int,
    end: int,
    rank: int = 4,
) -> list[Outcome]:
    """フレーム ``[start, end)`` の範囲のフレームごとに、
    melding で錬金したときの結果を返す (フレームの小さい順)。

    狙ったフレームの前後で錬金したら何が出るかを見るのに使う
    (本家の ``around`` を、錬金 1 回分のお守り全部に広げたもの)。
    1 個目だけなら ``o.charms[0]``。

    :param melding: 錬金のやり方
    :param start: 範囲の始まり
    :param end: 範囲の終わり (含まない)
    :param rank: 投入する素材のランク (1〜4。既定は最高ランク)
    :raises ValueError: rank が 1〜4 でない
    """
