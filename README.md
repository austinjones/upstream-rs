The mock HTTP upstream for development of HTTP applications and reverse proxies.

# Why use upstream?
- You need to run a local HTTP server, and you want 200 responses with minimal effort.
- You want to view the HTTP header and payload data that some component is emitting.
- You need to simulate responses with significant size or latency.
- You need responses for PUT/POST, so can't use `python -m http.server`

# Quickstart
From crates.io:
```
cargo install upstream
```

From git:
```
git clone https://github.com/austinjones/upstream-rs.git
cd upstream-rs
cargo install --path .
```

Then run with:
`upstream -p 8080`

## Features
- Upstream returns all requests with a generated JSON payload that contains a unique request ID.
- Upstream logs all HTTP requests to stdout.
- Upstream can introduce delay when returning HTTP headers, or additional delay for the HTTP body.
- Upstream can add data to HTTP headers or the body, so that responses have a desired size.
- Upstream binds to 127.0.0.1 by default, but can bind to all interfaces with the `--all-interfaces` flag

# Usage
```
$ upstream --help

Usage: upstream [OPTIONS]

Options:
  -p, --port <PORT>             binds to the specified port [default: 8080]
  -a, --all-interfaces          binds to all interfaces
  -q, --quiet                   suppresses output of incoming HTTP request data
      --delay-headers <MILLIS>  adds delay until HTTP headers are returned
      --delay-body <MILLIS>     adds delay until the HTTP body is returned
      --size-headers <BYTES>    generates a HTTP header with approximately the provided size
      --size-body <BYTES>       generates a HTTP body with approximately the provided size
  -h, --help                    Print help
  -V, --version                 Print version
```
