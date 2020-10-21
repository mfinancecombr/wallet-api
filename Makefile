setup:
	@rustup component add rustfmt
	@rustup component add clippy
	@type -p pre-commit >/dev/null 2>&1 || \
		(echo "Please install pre-commit and try again"; exit 1)
	@pre-commit install -f --hook-type pre-commit
	@pre-commit install -f --hook-type pre-push

mongo:
	@mkdir -p datadir
	@docker-compose up -d

kill-mongo:
	@docker stop wallet-api-mongo

run:
	@cargo +nightly run

test:
	@RUST_BACKTRACE=1 cargo +nightly test -vv -- --nocapture
