setup:
	@rustup component add rustfmt
	@rustup component add clippy
	@type -p pre-commit >/dev/null 2>&1 || \
		(echo "Please install pre-commit and try again"; exit 1)
	@pre-commit install -f --hook-type pre-commit
	@pre-commit install -f --hook-type pre-push

mongo:
	@docker run --name wallet-api-mongo --rm -p 27017:27017 \
		--mount type=bind,source=$(PWD)/datadir,target=/data/db -d mongo

kill-mongo:
	@docker stop wallet-api-mongo

run:
	@cargo +nightly run

test:
	@cargo +nightly test -vv
