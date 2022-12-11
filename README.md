# Ark

A busyness analyzer 🔧

> **Note**
> This projects is still in the development stage and will undergo heavy changes

## Development Environment

### Lang and tools

- [Rust lang and toolchain](https://www.rust-lang.org/tools/install)
- [Rust AWS Lambda build tool](https://www.cargo-lambda.info/guide/installation.html)

### Initializing all packages

#### Logger

Deploys to AWS Lambda and requires DynamoDB

Documentation in the future probably :)

#### Scanner

1. Copy .env.example to .env

It should look like this

```
MAC_ADDR_TIMEOUT_SECS=300
ARP_SCAN_PERIOD_SECS=1
MAC_CACHE_LOG_PERIOD_SECS=5
TRACE=true
RECONNECT_CMD="nmcli connection up SSID"
SCANNER_LOCATION=dev-location
```

Optionally, you can add LOG_API_URL=https://example.com and API_RETRY_LIMIT=3 if you have an API server that accepts the following:

```shell
curl -X POST 'https://example.com' -H 'Content-Type: application/json' -d '{ "location": "location", "device_count": 100 }'
```

2. Running the scanner

   > **Note**
   > Scanner does not work on Windows due to liminations of pnet_datalink dependency

   > **Note**
   > Scanner requires elevated permission to run due to [Layer 2 access](https://en.wikipedia.org/wiki/Data_link_layer)

**\*nix**

```shell
cd ark-scanner
sudo -E env "PATH=$PATH" cargo run -- ./.env
```
