# mhxx-rand の Python の API の説明 (Sphinx)。
# リポジトリ直下で make docs と打つと作る。
# 説明は mhxx_rand/__init__.pyi (型ヒントと docstring) から
# sphinx-autoapi で作るので、拡張モジュールのビルドは要らない

project = "mhxx-rand"
author = "skst328"
copyright = "2026, skst328"
language = "ja"

extensions = ["autoapi.extension"]

autoapi_type = "python"
autoapi_dirs = ["../mhxx_rand"]
# .py と .pyi が両方あれば .pyi を読む (中身は拡張モジュールの再公開だけなので)
autoapi_file_patterns = ["*.pyi", "*.py"]
autoapi_options = [
    "members",
    "undoc-members",
    "show-module-summary",
    "imported-members",
]
autoapi_add_toctree_entry = False
# 型ヒントを引数の並びの中に出す
autodoc_typehints = "signature"

html_theme = "furo"
html_title = "mhxx-rand"
