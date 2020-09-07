# Finance Wallet Rust

This is a small project for learning more Rust. It is a rewrite of [this project][mfinance-wallet-api-go] started in Go by my friend Marcelo `metal` Jorge Vieira.

## Building and running

This package uses Rocket, which currently only builds with Rust nightly. This is about to change but, for now, if you are using `rustup` make sure you have the nightly toolchain installed and run this on the project's directory:

```bash
rustup override set nightly
```

Otherwise make sure you always add `+nightly` when calling cargo. The following command will build and run the HTTP server:

```bash
cargo +nightly run
```

If all goes well you should now be able to look at the Swagger UI nicely provided by [rocket-okapi][okapi]:

http://localhost:8000/swagger-ui/

## Examples:

### Adding stock operations:

```curlrc
curl \
  http://localhost:8000/api/v1/stocks/operations \
  -X POST \
  -H 'Content-Type: application/json' \
  -d '{
    "portfolio": "default", "symbol": "PETR4", "type": "purchase", "broker": "Clear",
    "quantity": 500, "price": 10, "time": "2020-04-24T00:00:00Z", "fees": 0
    }'
```

```curlrc
curl \
  http://localhost:8000/api/v1/stocks/operations \
  -X POST \
  -H 'Content-Type: application/json' \
  -d '{
    "portfolio": "default", "symbol": "PETR4", "type": "sale", "broker": "Clear",
    "quantity": 500, "price": 10, "time": "2020-04-24T00:00:00Z", "fees": 0
    }'
```

### Obtaining the current position for a stock

```curlrc
curl \
  http://localhost:8000/api/v1/stocks/position/PETR4
```

[mfinance-wallet-api-go]: https://github.com/mfinancecombr/finance-wallet-api
[okapi]: https://github.com/GREsau/okapi
