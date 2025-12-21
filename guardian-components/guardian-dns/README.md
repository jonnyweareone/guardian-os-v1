# Guardian DNS

DNS filtering and safe browsing daemon for Guardian OS.

## Features

- **Domain Blocking** - Blocks adult content, malware, gambling sites
- **Safe Search Enforcement** - Forces safe search on Google, Bing, YouTube, DuckDuckGo
- **VPN/Proxy Detection** - Detects and blocks VPN, proxy, and DoH services
- **Query Logging** - Logs domains (not full URLs) with automatic cleanup

## Building

Requires Rust 1.70+:

```bash
# Build release binary
cargo build --release

# Or use the build script
./build.sh
```

## Configuration

Copy `config/dns.toml` to `/etc/guardian/dns.toml` and adjust as needed.

Key settings:

```toml
[server]
listen_addr = "127.0.0.1:53"  # Use 0.0.0.0:53 for network-wide
upstream_dns = ["1.1.1.3", "1.0.0.3"]  # Cloudflare Family

[safesearch]
enabled = true
google = true
bing = true
youtube = "moderate"  # "strict", "moderate", or "off"
duckduckgo = true

[blocking]
always_block = ["adult", "malware", "gambling"]
blocklists = [
    "https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts",
]

[vpn_detection]
enabled = true
block_vpn_domains = true
block_doh = true
```

## Running

Requires root (for port 53):

```bash
sudo ./target/release/guardian-dns
```

Or install as a systemd service:

```bash
sudo cp target/release/guardian-dns /usr/lib/guardian/
sudo cp config/guardian-dns.service /etc/systemd/system/
sudo systemctl enable guardian-dns
sudo systemctl start guardian-dns
```

## System Integration

### Set as system DNS resolver

```bash
# /etc/systemd/resolved.conf.d/guardian.conf
[Resolve]
DNS=127.0.0.1
FallbackDNS=
DNSStubListener=no
```

### Prevent DNS bypass (iptables)

```bash
# Redirect all DNS traffic to guardian-dns
sudo iptables -t nat -A OUTPUT -p udp --dport 53 ! -d 127.0.0.1 -j REDIRECT --to-port 53
sudo iptables -t nat -A OUTPUT -p tcp --dport 53 ! -d 127.0.0.1 -j REDIRECT --to-port 53
```

## API

Guardian DNS exposes a simple status API for integration with guardian-daemon:

- Blocklist count
- Query statistics
- Blocked attempts

## Testing

```bash
# Test DNS resolution
dig @127.0.0.1 google.com

# Test safe search (should resolve to forcesafesearch.google.com)
dig @127.0.0.1 www.google.com

# Test blocking (should return NXDOMAIN)
dig @127.0.0.1 nordvpn.com
```

## License

GPL-3.0 - See LICENSE file
