"""mhxx_rng.py の numba 版の検索を、print せずに位置の一覧で返す (比較用)。mhxx_rng.py には手を入れない。

show_fast / show_fast_multi と同じ準備をして、中の loop_*_nj を直接呼ぶ。返す位置は、show_fast が
「found!frame is…」で出す位置と同じ。
"""
import numpy as np

import mhxx_rng as m


def _state():
    # show_fast と同じ: 直近 7 個の乱数と、f を 0 にした乱数の状態
    return (m.r0, m.r1, m.r2, m.r3, m.r4, m.r5, m.r6), (m.x, m.y, m.z, m.w, m.t, 0)


def search_fast(start, step, funcname, p):
    """show_fast(start, step, funcname, p) で見つかる位置の一覧"""
    m.jump(start)
    if funcname == 'search':
        func = m.loop_search_nj
    elif funcname == 'search_greater':
        if p[2] == -1:
            func = m.loop_search_greater_skill1_nj
        elif p[0] == -1:
            func = m.loop_search_greater_skill2_nj
        else:
            func = m.loop_search_greater_nj
    else:
        raise ValueError(funcname)
    r, s = _state()
    found = func(step, r, s, np.array(p, dtype=np.int32), m.get_env())
    # show_fast は jump(i + f) した位置の f - 7 を出す。jump(start) の後の f は start + 7
    return [i + start + 7 for i in found]


def search_fast_multi(start, step, *p_list):
    """show_fast_multi(start, step, *p_list) で見つかる位置の一覧"""
    m.jump(start)
    if not p_list:
        return []
    r, s = _state()
    found = m.loop_search_greater_multi_nj(
        step, r, s, np.array(p_list, dtype=np.int32), m.get_env(), p_list[0][5])
    return [i + start + 7 for i in found]
