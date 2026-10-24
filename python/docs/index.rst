mhxx-rand
=========

MHXX の乱数と、マカ錬金のお守りの生成・検索のライブラリ。
`apmnnn/mhxx-rng <https://github.com/apmnnn/mhxx-rng>`_
の非公式な Rust 移植で、Python からは ``import mhxx_rand`` で使う。

使い方
------

風化したお守り (マカフシギ錬金) の「連撃4 痛撃5 スロット3」以上を、
位置 0〜1 億で探し、錬金のフレームに直す。

.. code-block:: python

   import mhxx_rand as mr

   want = mr.Charm(skill1=mr.Skill("連撃"), pts1=4,
                   skill2=mr.Skill("痛撃"), pts2=5, slots=3)
   # kind を省略すると、風化・古びたのどちらでも
   q = mr.Query(mr.Melding.Juju, want, kind=mr.Kind.Weathered)

   for f in mr.search(q, 0, 10**8, mr.Compare.AtLeast, nthreads=4):
       print("護石:", f.charm)
       for a in f.aimpoints():
           # 錬金のフレーム、何個目か、時間
           print(a.start, a.index, a.game_time())
       print("---")

狙ったフレームの前後で錬金したら何が出るかは、
:func:`mhxx_rand.outcomes` で見る。

.. code-block:: python

   for o in mr.outcomes(mr.Melding.Juju, 4713, 4734):
       print(o.frame, o.game_time(), o.charms)

鑑定画面で読んだ護石と何個目から、錬金したフレームを探すのは
:func:`mhxx_rand.search_meldings`。読んだ護石は種類が分からないので、
Query の kind は省略する。
何個もの護石を渡すと、全部が別の位置に出るフレームを探す。

.. code-block:: python

   targets = [mr.MeldingTarget(mr.Query(mr.Melding.Juju, want), 1)]
   for m in mr.search_meldings(targets, 0, 10**7):
       print(m.frame, m.indexes, m.game_time())
       # 4723 [[1]] 0d 0h 2m 37s 13f ...

錬金で出うるスキルと、第 1・第 2 スキルとして出るときのポイントの範囲は
:func:`mhxx_rand.skill_info` で引く (画面の選択肢を作るのに使う)。

.. code-block:: python

   for i in mr.skill_info(mr.Melding.Juju):
       print(i.skill.name, i.pts1, i.pts2)   # ガ性 (1, 7) (1, 5) など

調合の累計の個数から位置を探すのは :func:`mhxx_rand.search_combo`。
見つからなければ、途中の乱数のずれを診断する。

.. code-block:: python

   q = mr.ComboQuery([0, 2, 4, 6, 9, 12, 16, 19, 21, 24, 27, 30, 34, 36,
                      39, 42, 46, 50, 53, 56, 58, 62, 66, 68, 71, 74, 77,
                      79, 82, 85, 88, 90, 93, 96, 99])
   r = mr.search_combo(q, 0, 10**7)
   r.frames     # [] (この並びは途中で乱数がずれている)
   r.diagnosis  # Diagnosis(frame=4798, drifts=[...], total_drift=2, ...)

API
---

.. toctree::
   :maxdepth: 2

   autoapi/mhxx_rand/index
