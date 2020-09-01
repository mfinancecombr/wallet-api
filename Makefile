mongo:
	@docker run --name wallet-api-mongo --rm -p 27017:27017 \
		--mount type=bind,source=$(PWD)/datadir,target=/data/db -d mongo

kill-mongo:
	@docker stop wallet-api-mongo

run:
	@cargo +nightly run

test:
	@cargo +nightly test -vv
