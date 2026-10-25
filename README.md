# mhxx-rand

MHXX の乱数と、マカ錬金のお守りの生成・検索のライブラリ。
[apmnnn/mhxx-rng](https://github.com/apmnnn/mhxx-rng) の非公式な Rust 移植で、
Python からも使える。

## 使い方 (Python)

```python
import mhxx_rand as mr

charm = mr.Charm(skill1=mr.Skill("連撃"), pts1=4,
                 skill2=mr.Skill("痛撃"), pts2=5, slots=3)
q = mr.Query(mr.Melding.Juju, charm, kind=mr.Kind.Weathered)

for f in mr.search(q, 0, 10**8, mr.Compare.AtLeast, nthreads=4):
    print("護石:", f.charm)
    for a in f.aimpoints():
        # 錬金のフレーム、何個目か、時間
        print(a.start, a.index, a.game_time())
    print("---")

# 何通りもの護石をまとめて探す (f.index が qs の何番目か)
charms = [
    mr.Charm(skill1=mr.Skill("痛撃"), pts1=6, slots=3),
    mr.Charm(skill1=mr.Skill("連撃"), pts1=5, slots=3),
    mr.Charm(skill1=mr.Skill("特会"), pts1=6, slots=3),
]
qs = [mr.Query(mr.Melding.Juju, c) for c in charms]
for f in mr.search_many(qs, 0, 10**6, mr.Compare.AtLeast, nthreads=4):
    print(f.index, f.charm)   # 2 Charm(skill1=Skill("特会"), pts1=6, ...

# その範囲で錬金したら何が出るか
for o in mr.outcomes(mr.Melding.Juju, 4713, 4734):
    print(f"{o.frame}:")
    for k, c in o.charms:
        print(f"  {c}, {k}")   # 回量  5 KO　   5 S0, 風化したお守り

# 鑑定画面で読んだ護石 (種類は分からない) と何個目から、錬金のフレームを探す
targets = [mr.MeldingTarget(mr.Query(mr.Melding.Juju, charm), 1)]
for m in mr.search_meldings(targets, 0, 10**7):
    print(m.frame, m.game_time())   # 4723 0d 0h 2m 37s 13f ...

# 錬金で出うるスキルと、ポイントの範囲 (画面の選択肢に使う)
for i in mr.skill_info(mr.Melding.Juju):
    print(i.skill.name, i.pts1, i.pts2)

# 調合の累計の個数から位置を探す。
# 見つからなければ、途中の乱数のずれを診断する
q = mr.ComboQuery([0, 2, 4, 6, 9, 12, 16, 19, 21, 24, 27, 30, 34, 36,
                   39, 42, 46, 50, 53, 56, 58, 62, 66, 68, 71, 74, 77,
                   79, 82, 85, 88, 90, 93, 96, 99])
r = mr.search_combo(q, 0, 10**7)
print(r.frames)     # []
print(r.diagnosis)  # Diagnosis(frame=4798, drifts=[Drift(craft=17, ...
```

## 構成

| フォルダ | 中身 |
|---|---|
| `rand-core/` | 本体 (Rust) |
| `python/` | Python の入口 (pyo3) と API の説明 (Sphinx) |
| `reference/` | 比較用に、本家のノートブックを .py に切り出したもの |

## 開発

```bash
# Python の入口をビルドして .venv に入れる (Rust を変えたら作り直す)
uv sync
# いま有効にしている仮想環境に、編集可能な形で入れる (Rust を変えたら打ち直す)
make develop
# sdist と whl を dist/ に作る
make build
# Rust と Python のテスト
make test
# 本家から正解を作り直す (本家を写し直したときだけ)
make fixtures
# API の説明 (python/docs/_build/html)
make docs
```

## ライセンス

MIT License です。全文は [LICENSE](LICENSE) を参照してください。

### 使用しているソフトウェア

配布物に含まれるもの (と、移植したアルゴリズム) と、そのライセンスです。

| | ライセンス | 用途 |
|---|---|---|
| [PyO3](https://pyo3.rs/) | MIT または Apache-2.0 | Python への橋渡し |
| [Rayon](https://github.com/rayon-rs/rayon) | MIT または Apache-2.0 | 並列化 |
| [apmnnn/mhxx-rng](https://github.com/apmnnn/mhxx-rng) | MIT | 移植元 |
