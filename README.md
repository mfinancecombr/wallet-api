# Finance Wallet Rust

This is a small project for learning more Rust. It is a rewrite of
[this project][mfinance-wallet-api-go] started in Go by my friend
Marcelo `metal` Jorge Vieira.

## Building and running

This package uses Rocket, which currently only builds with Rust nightly. This
is about to change but, for now, if you are using `rustup` make sure you have
the nightly toolchain installed and run this on the project's directory:

```bash
rustup override set nightly
```

Otherwise make sure you always add `+nightly` when calling cargo. The following
command will build and run the HTTP server:

```bash
cargo +nightly run
```

## Doc

If all goes well you should now be able to look at the Swagger UI nicely
provided by [rocket-okapi][okapi]:

http://localhost:8000/swagger-ui/

## Examples

### Adding broker

```curlrc
curl 'http://localhost:8000/api/v1/brokers' \
  -X POST \
  -H 'Content-Type: application/json' \
  -d '{"name":"CLEAR"}'
```

### Adding portfolio

```curlrc
curl 'http://localhost:8000/api/v1/portfolios' \
  -X POST \
  -H 'Content-Type: application/json' \
  -d '{"name":"default"}'
```

### Adding stock events

```curlrc
curl 'http://localhost:8000/api/v1/events' \
  -X POST \
  -H 'Content-Type: application/json' \
  -d '{
      "eventType": "stock-operation",
      "time": "2020-10-05T00:00:00.000Z",
      "symbol": "BMGB4",
      "detail":{
          "price": 4,
          "quantity": 500,
          "fees": 0,
          "type": "purchase",
          "portfolios": ["PORTFOLIO-ID"],
          "broker": "BROKER-ID"
      }
  }'
```

### Obtaining the current position for a stock

```curlrc
curl http://localhost:8000/api/v1/stocks/position/PETR4
```

[mfinance-wallet-api-go]: https://github.com/mfinancecombr/finance-wallet-api
[okapi]: https://github.com/GREsau/okapi
