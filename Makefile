# 開発のコマンド。失敗するとそこで止まる (止めずに全部流すなら make -k)

# いま有効にしている仮想環境に、編集可能な形で入れる (その仮想環境に maturin が入っている前提)。
# Python のファイルの変更はすぐ効き、Rust を変えたら打ち直す。
# 仮想環境が有効でないと maturin はリポジトリの .venv に入れてしまうので、そのときは止める
develop:
	@test -n "$$VIRTUAL_ENV" || { echo "仮想環境が有効でない"; exit 1; }
	maturin develop --release -m python/Cargo.toml

# 配る形 (sdist と whl) を dist/ に作る。whl は sdist から作るので、sdist の中身の抜けもここで分かる
build:
	uv build --package mhxx-rand -o dist

# API の説明 (Sphinx) を python/docs/_build/html に作る
docs:
	uv run --group docs sphinx-build -b html python/docs python/docs/_build/html

# Rust と Python のテスト
test:
	cargo test
	uv run pytest python/tests

# 本家から正解 (rand-core/tests/fixtures/reference.json) を作り直す (本家を写し直したときだけ)
fixtures:
	uv run python reference/fixtures.py

.PHONY: develop build docs test fixtures
