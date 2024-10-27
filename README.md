# EchidnaLB
![EchidnaLB](./echidna-lb.png)
EchidnaLB is a layer 7 load-balancer for HTTP backend servers written in Rust.

## Features
- **HTTP1.1 & HTTP2**
  - Supports both HTTP/1.1 and HTTP/2 protocols, ensuring compatibility and performance for modern web applications.

- **IPv4 & IPv6 Listeners**
  - Supports both IPv4 & IPv6 listeners for greater network flexibility.

- **Multiple Load Balancing Algorithms**:
  - Round Robin
  - Weighted Round Robin
  - IP Hashing
  - Least Connections
  - Least Latency

- **Health Checks**:
  - Optional configurable health checks to monitor backend server status.

- **TLS Termination**:
  - Optional support for HTTPS with configurable SSL certificates.

## Installation

### Build yourself

Requirements:

- Git
- Rust Toolchain

To build the project yourself, clone the project and build it using `cargo`:

```sh
git clone https://github.com/echidna-lb/echidna-lb.git
cd echidna-lb

cargo build # or `cargo build --release` for better performance
sudo setcap 'cap_net_bind_service=+ep' target/debug/echidna-lb # (Optional) Allow to bind to port 80/443 without root

target/debug/echidna-lb --version
```

After that, you may want to add the binary to your `PATH`.

### Grab the binary

You may download a pre-compiled binary from the [GitHub release page](https://github.com/echidna-lb/echidna-lb/releases).

## Usage

By default, EchidnaLB binds to port 9000 (HTTP) and 9001 (HTTPS). If you wish to bind to the standard HTTP and HTTPS ports 80 and 443 respectively, on Linux, every port below 1024 is `privileged` so you either have to:

- run the program as root `sudo`
- use `setcap 'cap_net_bind_service=+ep' /path/to/echidna-lb`

### Configuration file

A minimal configuration file is required. Example:

```yaml
port: 9000
https_port: 9001 # optional
algorithm: "RoundRobin"
workers: 10 # optional

backends:
  - url: "http://127.0.0.1:8081"
    weight: 2 # optional
  - url: "http://127.0.0.1:8082"
    weight: 1 # optional

healthcheck: # optional
  interval_sec: 10 # optional
  route: "/health"

ssl: #optional
  cert_path: "cert.pem"
  key_path: "key.pem"
```

- `port`: The port on which the HTTP server listens. Defaults to `9000`.
- `https_port`: (Optional) The port on which the HTTPS server listens. Defaults to `9001`.
- `algorithm`: (Optional) The load balancing algorithm to use. Available options:
`RoundRobin`, `LeastConnections`, `WeightedRoundRobin`, `IPHashing` and `LeastLatency`. Defaults to `RoundRobin`.
- `workers`: (Optional) The number of worker threads to be used by the server. Defaults to `10`.
- `backends`: A list of backend servers with their URLs and weights (for weighted algorithms).
  - `url`: A backend endpoint URL.
  - `weight`: (Optional) The weight associated with the endpoint, only used for `WeightedRoundRobin` algorithm. Defaults to `1`.
- `healthcheck`: (Optional) Configuration for periodic health checks.
  - `route`: The HTTP route on the backend server used for health checks.
  - `interval_sec`: (Optional) Interval in seconds between health checks. Defaults to `10`.
- `ssl`: (Optional) SSL configuration for HTTPS.
  - cert_path: Path to the SSL certificate file.
  - key_path: Path to the SSL private key file.

### Command line arguments

- `--config <file>`: Path to the YAML configuration file.

Example:

```sh
echidna-lb --config config.yaml
```

## Contributing
Contributions are welcome! Please open an issue or submit a pull request if you have suggestions or improvements.

## License
This project is licensed under the MIT License.
See the [LICENSE](./LICENSE) file for more details.
