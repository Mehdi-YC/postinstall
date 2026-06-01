+++
title = "Networking Reference for Developers"
description = "What actually happens on the wire — layers, protocols, tools, and infrastructure a developer needs to understand"
[extra]
date = 2024-01-15
+++

# Networking Reference for Developers

*What actually happens on the wire — layers, protocols, tools, and infrastructure a developer needs to understand.*

## The Model: What a Dev Needs to Know

The OSI model has 7 layers. In practice, the internet collapses into roughly **4 functional layers** that matter to developers. You don't need the academic version — you need to know which layer a problem lives in so you can debug it.

| What You Call It | What It Does | Protocols | What You Debug Here |
|------------------|--------------|-----------|---------------------|
| **Application** | What your app sends/receives | HTTP, DNS, SSH, FTP, SMTP | HTTP status codes, malformed requests, wrong data format |
| **Transport** | Reliable delivery between apps | TCP, UDP | Connection refused, timeouts, connection reset |
| **Network / IP** | Addressing and routing between machines | IP, ICMP (ping), ARP | Host unreachable, no route to host, wrong IP |
| **Physical / Link** | Bits on the wire, frame delivery on a segment | Ethernet, WiFi, MAC | Cable unplugged, wrong VLAN, MAC filtering |

> **The layer determines where to look.** "Connection refused" is transport (TCP). "No route to host" is network (IP). "HTTP 400" is application (HTTP). Knowing the difference saves hours of debugging.

## Physical vs Logical: The Key Distinction

**Physical** = the actual cables, radio waves, electrical signals. A wire is a physical thing. A WiFi signal is a physical thing.

**Logical** = the addressing and organization built on top of physical connections. IP addresses, subnets, VLANs, ports, protocols — these are all logical constructs that exist in software/config, not in the wire itself.

    Physical layer:  cable/wireless carries electrical signals
    │
    Data link (logical):  signals grouped into frames with MAC addresses (Ethernet)
    │
    Network (logical):  frames grouped into packets with IP addresses
    │
    Transport (logical):  packets broken into segments with ports (TCP/UDP)
    │
    Application (logical):  segments carry your actual data (HTTP body, SQL query, etc.)

> **A switch operates at the data link layer.** It sees MAC addresses, not IP addresses. It delivers frames within a local network. A `switch port` is a physical jack. A `VLAN` is a logical partition of that switch — same physical cables, but logically isolated networks.

## What Happens When You Type a URL (Simplified)

    1. Browser parses "https://example.com/page" → extracts host, path
    │
    2. DNS lookup: example.com → 93.184.216.34
    │
    3. TCP handshake with 93.184.216.34:443 (3-way: SYN, SYN-ACK, ACK)
    │
    4. TLS handshake: negotiate encryption (if HTTPS)
    │
    5. HTTP request sent:
       GET /page HTTP/1.1
       Host: example.com
       Connection: keep-alive
    │
    6. Server processes request, runs app code, generates HTML
    │
    7. HTTP response sent:
       HTTP/1.1 200 OK
       Content-Type: text/html
       Content-Length: 1234

       <html>...</html>
    │
    8. Browser renders the page

Steps 2-4 happen in milliseconds. Step 5-7 is what your code controls. If something fails at step 2, it's a DNS problem. Step 3, it's network/firewall. Step 5-7, it's your app or the server.

## Frames, Packets, Segments, Messages

These are all names for "data on the wire" at different layers:

| Name | Layer | Contains | Addressed By |
|------|-------|----------|---------------|
| **Frame** | Data link (Ethernet) | Packet + MAC addresses + checksum | MAC address (AA:BB:CC:DD:EE:FF) |
| **Packet** | Network (IP) | Segment + source/dest IP + TTL | IP address (192.168.1.1) |
| **Segment** | Transport (TCP/UDP) | Your data + source/dest port + sequence numbers | Port (80, 443, 5432) |
| **Message** | Application (HTTP) | Headers + body (the actual content) | URL path (/page) |

> **Nesting: Frame wraps Packet wraps Segment wraps Message.** When you send an HTTP request, it gets wrapped in a TCP segment, which gets wrapped in an IP packet, which gets wrapped in an Ethernet frame. Each layer adds its own header. This is called **encapsulation**.

## MAC Address vs IP Address

| | MAC Address | IP Address |
|--|-------------|-------------|
| **Layer** | Data link (2) | Network (3) |
| **Format** | AA:BB:CC:DD:EE:FF (48-bit, hex) | 192.168.1.1 (32-bit, decimal) |
| **Scope** | Local network segment only | Global (routable) |
| **Assigned by** | Hardware manufacturer (burned in) | Network admin / DHCP |
| **Changes?** | No (per interface) | Yes (different network = different IP) |
| **What a switch sees** | MAC addresses | Nothing (switches don't look at IP) |
| **What a router sees** | Nothing (routers strip MAC, add new one) | IP addresses |

> **MAC addresses don't cross routers.** When a packet goes through a router, the router strips the source/dest MAC from the frame and adds new ones pointing to the next hop. MAC addresses are only meaningful on a local network segment. This is why you can't find someone's MAC address from across the internet.

## TCP (Transmission Control Protocol)

TCP is **reliable**. It guarantees that data arrives intact, in order, and without duplication. It does this through:

- **Handshake:** 3-way exchange before data flows
- **Sequence numbers:** Every byte is numbered so the receiver can reorder and detect gaps
- **Acknowledgments (ACKs):** Receiver tells sender "I got bytes up to N"
- **Retransmission:** If sender doesn't get an ACK within a timeout, it resends
- **Flow control:** Receiver tells sender to slow down if it's overwhelmed
- **Connection teardown:** 4-way handshake to close gracefully

### The TCP Handshake (3-Way)

    Client → SYN (I want to talk, my sequence starts at X)
    │
    Server → SYN-ACK (OK, I acknowledge X, my sequence starts at Y)
    │
    Client → ACK (I acknowledge Y)
    │
                === connection established ===
    │
    Data flows in both directions...
    │
    Either side → FIN (I'm done sending)
    │
    Other side → ACK + FIN (OK, I'm done too)
    │
    First side → ACK (confirmed)
    │
                === connection closed ===

> **"Connection refused" = the server sent a RST instead of SYN-ACK.** Either nothing is listening on that port, or a firewall rejected it. "Connection timed out" = no response at all (firewall dropping packets, or host unreachable).

### TCP State Machine (The Important Parts)

| State | Meaning |
|-------|---------|
| `LISTEN` | Server is waiting for connections on a port |
| `SYN_SENT` | Client sent SYN, waiting for SYN-ACK |
| `ESTABLISHED` | Handshake complete, data can flow |
| `FIN_WAIT_1` | Initiated close, waiting for ACK |
| `FIN_WAIT_2` | Sent FIN + got ACK, waiting for other side's FIN |
| `CLOSE_WAIT` | Got other side's FIN, haven't closed yet (app hasn't called close()) |
| `TIME_WAIT` | Closed, waiting 2MSL (~60s) to ensure the other side got the ACK |
| `CLOSED` | Connection fully closed |

> **`TIME_WAIT` is normal, not an error.** After closing a connection, the side that initiated the close stays in TIME_WAIT for ~60 seconds. It prevents old packets from a previous connection from being confused with a new one. Thousands of TIME_WAIT connections can exhaust ports — this is why connection reuse and keep-alive matter.

## UDP (User Datagram Protocol)

UDP is **unreliable**. No handshake, no acknowledgments, no ordering, no retransmission. It just sends data and hopes it arrives. So why use it?

- **Speed:** No overhead from handshakes or ACKs. Lower latency.
- **Simplicity:** No connection state to manage.
- **Broadcast/multicast:** TCP can't do this. UDP can send to multiple recipients.
- **Tolerates loss:** DNS queries, metrics, streaming — losing a packet is fine.

| Protocol | Transport | Why |
|----------|-----------|-----|
| HTTP/1.1 | TCP | Reliable page delivery matters |
| HTTP/2, HTTP/3 | TCP / UDP (QUIC) | HTTP/3 uses UDP (QUIC) for faster connection setup |
| DNS | UDP | Small queries, retry if no response is faster than TCP handshake |
| SSH | TCP | Reliable shell session |
| Video streaming | UDP | Dropped frames = momentary glitch, not a broken page |
| Databases (PostgreSQL, MySQL) | TCP | Data integrity is non-negotiable |
| Logging/metrics | UDP | Losing a log line is acceptable |
| VPN (WireGuard) | UDP | Implements its own reliability on top of UDP |

## Ports

Ports are **transport-layer addresses**. They identify which application on a machine should receive the data. They range from 0 to 65535.

| Range | Name | Examples |
|-------|------|----------|
| `0-1023` | Well-known (need root/admin) | 80 (HTTP), 443 (HTTPS), 22 (SSH), 53 (DNS), 25 (SMTP) |
| `1024-49151` | Registered (assigned to specific apps) | 3306 (MySQL), 5432 (PostgreSQL), 6379 (Redis), 8080 (HTTP alt) |
| `49152-65535` | Ephemeral (temporary, auto-assigned) | Your outgoing connections use these |

> **A port is just a number in a header field.** It's not a physical thing. When you say "port 80 is open," it means "a program on this machine is listening for TCP segments addressed to port 80." Multiple programs can use port 80 on different IP addresses (different interfaces).

> **"Address already in use" means another process is already bound to that port.** Use `ss -tlnp | grep :8000` to find what's using it. `kill <PID>` to stop it. Or use a different port.

### How Your OS Handles Connections

    # A TCP connection is defined by 4 things (a socket tuple):
    #   source IP + source port + dest IP + dest port
    # Same dest IP+port, different source port = different connection
    # This is why your browser can open 6 connections to the same server at once

    # Only ONE process can bind to a specific local IP+port at a time.
    # That's why you can't start two apps on port 8000.

## What Your Code Actually Controls

    # When you write this in Python:
    socket.connect(("93.184.216.34", 443))

    # Your OS:
    # 1. Picks a random ephemeral source port (e.g., 51234)
    # 2. Sends SYN to 93.184.216.34:443 with source 192.168.1.100:51234
    # 3. Handles the TCP handshake, ACKs, retransmissions for you
    # 4. Delivers the data you write() to the transport layer
    # 5. Wraps it in IP packet with dest 93.184.216.34, source 192.168.1.100
    # 6. Wraps that in an Ethernet frame with your router's MAC as dest

    # When you write:
    server = socket.socket()
    server.bind(("0.0.0.0", 8000))

    # Your OS:
    # 1. Tells the network stack: "accept TCP segments for port 8000"
    # 2. When a SYN arrives, responds with SYN-ACK (enters SYN_RCVD state)
    # 3. When ACK comes back, moves to ESTABLISHED
    # 4. Calls your accept() callback

## HTTP Overview

HTTP is a **text-based protocol** that runs on top of TCP. It's just formatted text sent over a TCP connection. That's it. No magic. Every HTTP message is either a request (client to server) or a response (server to client), and both follow the same basic structure.

### HTTP Request Structure

What actually gets sent on the wire:
```bash
    # Request line
    GET /api/users?page=2&limit=50 HTTP/1.1
    # ↑ method  ↑ path+query       ↑ version

    # Headers (key: value, one per line)
    Host: example.com
    Accept: application/json
    Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6...
    Content-Type: application/json
    User-Agent: Mozilla/5.0
    Accept-Encoding: gzip, deflate
    Connection: keep-alive

    # Empty line separates headers from body

    # Body (optional - present in POST/PUT/PATCH, absent in GET/DELETE)
    {"name": "Alice", "email": "alice@example.com"}
```
> **The empty line between headers and body is mandatory.** Without it, the server can't tell where headers end and body begins. If you get "400 Bad Request" and can't figure out why, check for a missing blank line between headers and body.

### HTTP Response Structure

What the server sends back:
```bash
    # Status line
    HTTP/1.1 200 OK
    # ↑ version  ↑ status code  ↑ reason phrase

    # Headers
    Content-Type: application/json
    Content-Length: 47
    Cache-Control: max-age=3600
    Server: nginx/1.24
    X-Request-Id: abc-123-def

    # Empty line

    # Body
    {"id": 1, "name": "Alice"}
```
### HTTP Methods (Verbs)

| Method | Has Body? | Idempotent? | Safe? | Purpose |
|--------|-----------|-------------|-------|---------|
| `GET` | No | Yes | Yes | Retrieve a resource |
| `POST` | Yes | No | No | Create a resource (or trigger an action) |
| `PUT` | Yes | Yes | No | Replace a resource entirely (full update) |
| `PATCH` | Yes | No | No | Partial update (change only the fields you send) |
| `DELETE` | No | Yes | No | Delete a resource |
| `HEAD` | No | Yes | Yes | Like GET but only returns headers (check if exists, get Content-Length) |
| `OPTIONS` | No | Yes | Yes | Ask server what methods are allowed (CORS preflight) |

> **Idempotent** = doing it once has the same effect as doing it 100 times. `GET` the same page 10 times = same result. `POST` create 10 orders = 10 different orders. This matters for retry logic: if a request fails mid-flight, safe to retry idempotent methods.

> **GET with a body is technically allowed by the spec but practically never used.** Proxies and caches may strip it. Use `POST` if you need to send data, even if you're just retrieving.

### Status Codes

| Range | Meaning | Common Codes |
|-------|---------|---------------|
| **1xx** | Informational | 100 Continue, 101 Switching Protocols |
| **2xx** | Success | 200 OK, 201 Created, 204 No Content |
| **3xx** | Redirection | 301 Moved Permanently, 302 Found, 304 Not Modified |
| **4xx** | Client error (your fault) | 400 Bad Request, 401 Unauthorized, 403 Forbidden, 404 Not Found, 405 Method Not Allowed, 413 Payload Too Large, 429 Too Many Requests, 422 Unprocessable Entity |
| **5xx** | Server error (their fault) | 500 Internal Server Error, 502 Bad Gateway, 503 Service Unavailable, 504 Gateway Timeout |

### The Ones That Confuse Developers

| Code | What It Actually Means |
|------|------------------------|
| `400` | Your request is malformed (bad JSON, missing required header, syntax error in URL). Read the response body for details. |
| `401` | "I don't know who you are." Missing or invalid authentication (wrong token, expired session). |
| `403` | "I know who you are, but you're not allowed." Authentication OK, authorization denied. Wrong role, wrong permissions. |
| `404` | "This URL doesn't exist on this server." Route not found. Not "server not found" (that's DNS). |
| `405` | "You used the wrong HTTP method." GET on an endpoint that only accepts POST. Or POST on one that only accepts GET. |
| `413` | "Your request body is too big." Usually a server config limit (nginx default: 1MB). |
| `422` | "I understood your request but the data is wrong." Validation error: missing required field, wrong data type, business rule violation. |
| `429` | "You're sending too many requests." Rate limiting. Slow down. |
| `500` | "Something broke on our end." Unhandled exception in the app code. Check server logs. |
| `502` | "The server in front of me is broken." Nginx got an invalid response from the app. App is down or crashed. |
| `503` | "The server is overloaded or down for maintenance." App isn't accepting connections yet. |
| `504` | "The server in front of me timed out waiting for the app." App took too long to respond. |
| `301 vs 302` | `301`: "Moved permanently, update your URL." `302`: "Moved temporarily, keep using the old URL." Browsers cache 301s by default. Use 302 for POST redirects (to prevent method change to GET). |
| `304` | "Nothing changed since last time you asked." Your cached version is still valid. No body returned. |

### Headers That Actually Matter

| Header | Direction | What It Does |
|--------|-----------|---------------|
| `Host` | Request | The domain name. **Mandatory in HTTP/1.1.** Without it, the server can't route the request to the right virtual host. "400 Bad Request" if missing. |
| `Content-Type` | Both | MIME type of the body. `application/json`, `text/html`, `multipart/form-data`. **Mandatory** when there's a body. |
| `Content-Length` | Response | Body size in bytes. Lets the client know when the response is complete. Replaced by `Transfer-Encoding: chunked` for streaming. |
| `Authorization` | Request | Credentials. Usually `Bearer <token>` for JWT, or `Basic base64(user:pass)` for simple auth. |
| `Accept` | Request | "I can understand these formats." `application/json`, `text/html`. Server picks the best match. |
| `Cache-Control` | Response | "You can cache this for N seconds." `max-age=3600`, `no-cache`, `no-store`, `private`. |
| `Set-Cookie` | **Response only** | Tell the browser to store a cookie. **Cannot be set by JavaScript.** This is why you can't read `HttpOnly` cookies from `document.cookie`. |
| `Access-Control-Allow-Origin` | **Response only** | CORS: which origins can call this API. `*` = any origin. **Never use `*` with credentials.** |
| `Access-Control-Allow-Headers` | **Response only** | CORS: which request headers are allowed in preflight. |
| `X-Forwarded-For` | Request | Original client IP when behind a proxy. `X-Forwarded-For: client_ip, proxy1_ip`. Trust chain. |
| `X-Request-Id` | Response | Unique ID for the request. Essential for tracing issues through logs across services. |
| `User-Agent` | Request | Browser/app identification string. Used by servers for analytics and compatibility. |
| `Connection` | Both | `keep-alive` = reuse the TCP connection for multiple requests. `close` = close after one request. |
| `Transfer-Encoding` | Response | `chunked` = send body in chunks (no need to know Content-Length upfront). Enables streaming. |

### HTTP/1.1 vs HTTP/2 vs HTTP/3

| | HTTP/1.1 | HTTP/2 | HTTP/3 |
|--|----------|--------|--------|
| **Transport** | TCP | TCP | UDP (QUIC) |
| **Connections** | 1 request per TCP connection (or pipelined) | Multiplexed: many requests over one TCP connection | Multiple streams over one UDP "connection" |
| **Headers** | Full headers repeated every request | HPACK compression: headers sent once, referenced after | QPACK: similar to HPACK |
| **Ordering** | Sequential (head-of-line blocking) | Streams can interleave freely | Same as HTTP/2 |
| **TLS** | Separate handshake (2 RTTs added) | Embedded (saves ~1 RTT) | Embedded (same as HTTP/2) |
| **Connection setup** | TCP handshake (1 RTT) + TLS (1-2 RTTs) = 2-3 RTTs | TCP + TLS = 2-3 RTTs | TLS (no TCP) = 1 RTT |
| **Head-of-line blocking** | Yes: browser waits for one response before sending next request on same connection | No: multiplexed, no HOL blocking | No |

> **HTTP/3 is not "HTTP over UDP" in the simple sense.** It uses QUIC, which adds its own reliability, ordering, and congestion control on top of UDP. Applications still see an HTTP-like interface. The complexity is pushed into the library, not your code.

### Keep-Alive and Connection Reuse

    # HTTP/1.1 default: keep-alive is ON
    # Without keep-alive (Connection: close):
    #   Request 1: TCP handshake (1 RTT) + HTTP request/response (1 RTT) = 2 RTTs
    #   Request 2: TCP handshake + HTTP = 2 RTTs
    #   Request 3: same = 2 RTTs
    #   Total: 6 RTTs for 3 requests

    # With keep-alive (Connection: keep-alive):
    #   Request 1: TCP handshake (1 RTT) + HTTP request/response = 2 RTTs
    #   Request 2: just HTTP = 1 RTT
    #   Request 3: just HTTP = 1 RTT
    #   Total: 4 RTTs for 3 requests

    # HTTP/2/3: all requests multiplexed over ONE connection:
    #   Setup: 1 RTT (HTTP/3) or 2-3 RTTs (HTTP/2)
    #   Requests 1-3 (and hundreds more): 1 RTT each (all in parallel if server allows)
    #   Total: 2-4 RTTs for 3 requests

### Form Data: Content-Type Matters

application/x-www-form-urlencoded (default for HTML forms):

    # What your browser sends when you submit <form> without enctype
    POST /submit HTTP/1.1
    Content-Type: application/x-www-form-urlencoded
    Content-Length: 27

    name=Alice&email=alice%40example.com&age=30
    # URL-encoded: spaces become +, special chars become %XX

multipart/form-data (for file uploads):

    # What your browser sends when <form enctype="multipart/form-data">
    POST /upload HTTP/1.1
    Content-Type: multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW
    Content-Length: 342

    ------WebKitFormBoundary7MA4YWxkTrZu0gW
    Content-Disposition: form-data; name="file"; filename="photo.jpg"
    Content-Type: image/jpeg

    [binary data]
    ------WebKitFormBoundary7MA4YWxkTrZu0gW
    Content-Disposition: form-data; name="description"

    A photo of Alice
    ------WebKitFormBoundary7MA4YWxkTrZu0gW--

application/json (APIs):

    POST /api/users HTTP/1.1
    Content-Type: application/json
    Content-Length: 47

    {"name": "Alice", "email": "alice@example.com"}
    # No URL encoding needed — JSON is the raw body, not form fields

> **Wrong Content-Type = wrong parsing.** Sending JSON with `application/x-www-form-urlencoded` (or no Content-Type) will make the server try to parse JSON as form fields, which silently fails. Always set Content-Type when you have a body.

## What DNS Does

DNS translates **domain names** (human-readable) into **IP addresses** (machine-readable). It's the phonebook of the internet. Without it, every URL would be `http://93.184.216.34/`.

### The Resolution Process

    1. Browser cache → "Have I looked up example.com recently?" (check TTL)
    If not found ↓
    │
    2. OS cache → check /etc/hosts, nscd, systemd-resolved
    If not found ↓
    │
    3. Recursive resolver (usually your ISP's DNS, e.g., 8.8.8.8 or 1.1.1.1)
       ├─ Check its own cache
       ├─ If cached: return IP immediately
       └─ If not: continue ↓
    │
    4. Root nameserver → "Who handles .com?"
       Returns the address of .com's authoritative servers
    │
    5. .com TLD server → "Who handles example.com?"
       Returns the address of example.com's nameserver (usually your DNS provider's)
    │
    6. Authoritative nameserver → "What is example.com?"
       Returns: 93.184.216.34
    │
    Result is cached at every level for the TTL duration

> **DNS lookups typically take 20-120ms.** But they're cached aggressively. Once resolved, subsequent requests are instant. The TTL (Time To Live) in the DNS record controls cache duration. Low TTL = fresh data, more lookups. High TTL = stale data, fewer lookups.

### Record Types

| Type | Points To | Example |
|------|-----------|---------|
| `A` | IPv4 address | `example.com → 93.184.216.34` |
| `AAAA` | IPv6 address | `example.com → 2606:2800:220:1:248:1893:25c8:1946` |
| `CNAME` | Alias (redirects one domain to another) | `www.example.com → example.com` |
| `MX` | Mail server | `example.com → mail.example.com` (with priority) |
| `NS` | Nameserver (who is authoritative for this domain) | `example.com → ns1.dnsprovider.com` |
| `TXT` | Arbitrary text (SPF, verification, etc.) | `"v=spf1 include:_spf.google.com ~all"` |
| `SRV` | Service (hostname + port) | `_sip._tcp.example.com → sip.example.com:5060` |
| `PTR` | Reverse DNS (IP → name) | `93.184.216.34 → example.com` |
| `SOA` | Start of Authority (zone metadata) | Primary NS, admin email, serial, refresh, retry, expire, minimum TTL |

### Common DNS Tools
```bash
    # Query specific record type
    dig example.com A                    # IPv4 address
    dig example.com AAAA                 # IPv6 address
    dig example.com MX                   # mail servers
    dig example.com TXT                  # text records (SPF, verification)

    # Trace the full resolution path (see every step)
    dig +trace example.com

    # Query a specific DNS server
    dig @1.1.1.1 example.com A

    # Short output
    dig +short example.com A

    # Reverse DNS (IP → name)
    dig -x 93.184.216.34 +short

    # Check DNS propagation after a change
    dig example.com @8.8.8.8 +short

    # Alternative tools
    nslookup example.com                # simpler output, fewer options
    host -t A example.com               # another alternative

    # Check your DNS resolver
    cat /etc/resolv.conf
    nmcli dev show | grep DNS           # NetworkManager DNS (Linux)

    # Flush local DNS cache (macOS)
    sudo dscacheutil -flushcache
    sudo discoveryutil mdnsflushcache

    # Flush local DNS cache (Linux with systemd-resolved)
    sudo systemd-resolve --flush-caches

    # Flush local DNS cache (Linux with nscd)
    sudo nscd -i hosts

    # Flush browser DNS: clear browsing data or use devtools > Network > Disable cache (temporarily)
```
### DNS Propagation

When you change a DNS record, it doesn't take effect everywhere instantly. Propagation happens gradually:

- **Authoritative server:** Updated immediately (TTL clock starts ticking)
- **Recursive resolver (ISP):** Updates after current TTL expires (up to the old TTL value, usually 300-3600 seconds for common records)
- **Browser cache:** Respects TTL but may override with heuristics
- **OS cache:** Follows TTL

> **Set low TTLs (300s) before making changes.** If your TTL is 86400 (1 day), you'll wait up to a day for changes to propagate. Set it to 300 a day before, then make the change, then set it back to 86400 after confirming it works.

### Common DNS Problems

| Symptom | Likely Cause |
|---------|---------------|
| Works with IP, fails with domain | DNS not resolving. Check with `dig`. Check propagation. |
| Works on one device, not another | DNS cache on the failing device. Flush it. |
| "Server Not Found" (NXDOMAIN) | Domain doesn't exist at all, or a typo |
| "Server Failure" (SERVFAIL) | The authoritative server is misconfigured or down |
| "Refused" (REFUSED) | The authoritative server refuses to answer (wrong name server, no access) |
| "No answer" (NOERROR, empty response) | The domain exists but has no records of the queried type (e.g., no AAAA records) |
| Sporadic failures | TTL expiring on one resolver but not another. Inconsistent propagation. |
| Works in browser, fails in curl | curl doesn't use the system DNS. Use `curl --resolve` or check `/etc/resolv.conf`. |
| CNAME loop | Domain A → Domain B → Domain A. Cannot resolve. |

## What TLS Does

TLS (Transport Layer Security) encrypts the data between your browser and the server so eavesdroppers on the network can't read it. That's it. It doesn't make your app secure — it protects data **in transit**.

> **HTTPS = HTTP over TLS over TCP.** The data on the wire is encrypted, but the server and browser see plaintext. TLS is between transport endpoints, not end-to-end encryption.

### The TLS Handshake (What Actually Happens)

    Client Hello → I want to talk HTTPS. Here are TLS version + cipher suites I support + random number (ClientHello)
    │
    Server Hello → OK, using TLS 1.3, this cipher suite, my certificate, and random number (ServerHello)
    │
    Server Certificate → Here's my certificate proving I am example.com (signed by a CA your browser trusts)
    │
    Client validates → "Is this cert valid? Not expired? Correct domain? Signed by a trusted CA?"
    │
    Key Exchange → Client and server agree on a shared secret without sending it in the clear
      (using the server's public key in the certificate)
    │
                === encryption established ===

    Application data is now encrypted before being sent

### Certificates & Chain of Trust

    # Certificate contains:
    # - Domain name (e.g., example.com)
    # - Public key of the server
    # - Who issued it (the CA)
    # - Validity period (not before / not after)
    # - Signature from the CA (cryptographic proof the CA issued it)

    # Chain of trust:
    # Browser trusts Root CAs (built into OS/browser)
    # Root CA signs Intermediate CA certificates
    # Intermediate CA signs your certificate
    # Browser validates: your cert → intermediate → root (must be unbroken chain)

    # Self-signed cert: you sign it yourself. Browsers don't trust it by default.
    # Fine for dev/testing. Never use in production.

### Common TLS Issues

| Symptom | Likely Cause |
|---------|---------------|
| "Your connection is not private" (NET::ERR_CERT_AUTHORITY_INVALID) | Self-signed cert, expired cert, cert for wrong domain, or cert signed by untrusted CA |
| "Certificate has expired" | Cert's not-after date has passed. Renew it. |
| "Hostname mismatch" | Cert is for `api.example.com` but you're connecting to `staging.example.com` |
| "Mixed content" warning | HTTPS page loading HTTP resources (images, scripts). Browser blocks or warns. |
| Cert works in browser, fails in curl/Python | curl/Python don't use the OS certificate store. Use `curl -k` (insecure, but confirms DNS/TCP works) or set `REQUESTS_CA_BUNDLE` in Python. |

### Certificate Types (Let's Encrypt vs Commercial)

| | Let's Encrypt | Commercial (DigiCert, etc.) |
|--|---------------|----------------------------|
| **Cost** | Free | $50-$2000+/year |
| **Validity** | 90 days max (must renew frequently) | 1-3 years |
| **Automation** | Designed for full automation (certbot) | Often requires manual steps |
| **Wildcard certs** | Supported (via DNS-01 challenge) | Supported |
| **EV (green bar in browser)** | Not offered | Available (requires business verification) |
| **Support** | Community (forums, docs) | Dedicated support |

### Renewal and Expiry
```bash
    # Check when a cert expires:
    echo | openssl s_client -connect example.com:443 2>/dev/null | openssl x509 -noout -enddate -startdate

    # Let's Encrypt renewal (via certbot):
    # certbot automatically handles this with a cron/systemd timer
    sudo certbot renew --dry-run          # test renewal without making changes
    sudo certbot renew                 # actually renew
    # certbot installs a timer that auto-renews when cert is 30 days old
```
### Mixed Content

    # HTTPS page loading HTTP resource → browser blocks or warns
    # Common causes:
    # <img src="http://example.com/img.jpg">           ← blocks the image
    # <script src="http://cdn.example.com/lib.js">   ← blocks the script
    # <iframe src="http://...">                     ← blocks the iframe

    # Fix: use https:// for ALL resources, or upgrade-insecure-requests in CSP
    # Or for dev: chrome://flags/#allow-insecure-localhost

## Tools

### curl
```bash
    # Basic GET (same as opening in a browser)
    curl https://api.example.com/users

    # Verbose (see the full request and response including headers)
    curl -v https://api.example.com/users

    # Show only response headers
    curl -I https://api.example.com/users

    # Silent mode (no progress bar)
    curl -s https://api.example.com/users

    # Follow redirects
    curl -L https://example.com/nonexistent-page

    # Download a file
    curl -O https://example.com/file.zip
    curl -o myfile.zip https://example.com/file.zip  # custom filename

    # POST with JSON
    curl -X POST https://api.example.com/users \
      -H "Content-Type: application/json" \
      -d '{"name":"Alice","email":"alice@example.com"}'

    # POST with form data
    curl -X POST https://example.com/login -d "username=alice&password=secret"

    # POST from a file
    curl -X POST -H "Content-Type: application/json" -d @data.json https://api.example.com/users

    # Add auth header
    curl -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..." https://api.example.com/me

    # Measure response time
    curl -w "\nTime: %{time_total}s\n" -o /dev/null -s https://example.com

    # Show only status code
    curl -o /dev/null -s -w "%{http_code}\n" https://example.com

    # Show response headers + body
    curl -i https://api.example.com/users

    # Ignore SSL certificate errors (only for debugging!)
    curl -k https://self-signed.example.com

    # Resolve using a specific DNS server
    curl --resolve "example.com:443:127.0.0.1" https://example.com

    # Send a cookie
    curl -b "session=abc123" https://example.com/dashboard

    # Save cookies to a file, then use them in next request
    curl -c cookies.txt -X POST https://example.com/login -d "user=alice"
    curl -b cookies.txt https://example.com/dashboard

    # Upload a file
    curl -F "file=@localfile.jpg" https://example.com/upload
    curl -F "file=@localfile.jpg;type=image/jpeg" https://example.com/upload
    curl -F "file=@localfile.jpg;filename=photo.jpg" https://example.com/upload
```
### netcat (nc)
```bash
    # Listen on a port and respond manually
    nc -l -p 8080
    # (type HTTP response manually, Ctrl+C to close)

    # Connect to a port (test if something is listening)
    nc -v example.com 80
    # If connected: something is listening. If refused: nothing there.

    # Connect and send raw HTTP
    printf "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n" | nc example.com 80

    # Port scan a single port range
    nc -zv example.com 80 443 8000-8010
    # -z: scan without sending data (connect-only)
    # -v: verbose (shows open/closed/filtered)

    # Simple chat between two machines
    # Machine A: nc -l 9999
    # Machine B: nc machine-a-ip 9999
    # Both can now type and see each other's text

    # Test if a port is open locally
    nc -z localhost 8000
```
### ss (Socket Statistics)
```bash
    # All listening TCP ports
    ss -tlnp

    # All TCP connections (listening + established)
    ss -tamp

    # Filter by port
    ss -tlnp | grep :5432

    # Filter by process
    ss -tlnp | grep python

    # All listening sockets (TCP + UDP)
    ss -tulnp

    # Show process info
    ss -tulnp -p
```
### tcpdump (Packet Capture)
```bash
    # Capture all traffic on eth0
    sudo tcpdump -i eth0

    # Capture only port 80 (HTTP)
    sudo tcpdump -i eth0 port 80

    # Capture DNS queries
    sudo tcpdump -i any port 53

    # Save capture to file
    sudo tcpdump -i eth0 -w capture.pcap

    # Read a capture file
    tcpdump -r capture.pcap

    # Human-readable output
    sudo tcpdump -i eth0 -A -nn

    # Filter by host
    sudo tcpdump -i eth0 host example.com

    # Filter by source
    sudo tcpdump -i eth0 src host 192.168.1.100

    # Combination filter
    sudo tcpdump -i eth0 "host example.com and port 443"

    # -i = interface
    # -A = print ASCII text alongside hex
    # -nn = no name resolution (faster)
    # -w file = write to file
    # -c count = stop after N packets
```
### traceroute / tracepath
```bash
    # Trace the route packets take to reach a host
    traceroute example.com      # Linux/macOS (uses UDP by default)
    tracepath example.com        # modern Linux (uses ICMP)

    # Output: each hop shows the router/intermediate node and the round-trip time to reach it
    # If you see * * * * at some hop, that hop blocks ICMP (common for cloud providers)
```
### wireshark

Wireshark is a GUI packet analyzer. It does what tcpdump does but with a proper interface.

Open it, select your network interface, start capture, do the action, stop capture, then inspect.

Key features:
- Color-coded protocol dissection (HTTP, TCP, DNS, TLS are highlighted differently)
- "Follow TCP stream" to see the full conversation between two endpoints
- Filters: "http" shows only HTTP, "tcp.port == 5432" shows only PostgreSQL traffic
- "Follow TCP stream" → right-click a packet → Follow → see full conversation
- Statistics → Conversations → shows all TCP streams
- Analyze → Expert Info → automated problem detection

Export → "Export specified packets" → save as .pcap for later analysis or sharing

## Infrastructure

### Switch

A switch connects devices on a **local network (LAN)**. It operates at the data link layer (layer 2) and uses **MAC addresses** to decide where to send frames. It builds a table mapping MAC addresses to ports, and learns which device is on which port by watching source MAC addresses in incoming frames.

    # What a switch actually does with a frame:
    # Frame arrives on port 1 from MAC AA:BB:CC:DD:EE:01, dest MAC FF:FF:FF:FF:FF:FF
    # Switch looks up FF:FF:FF:FF:FF:FF → it's on port 5
    # Switch forwards frame out port 5
    # It does NOT look at IP addresses. The IP inside the frame is irrelevant to the switch.

    # Broadcast frame (dest MAC FF:FF:FF:FF:FF:FF:FF):
    # Switch forwards to ALL ports (except the one it arrived on)
    # This is how ARP ("Who has 192.168.1.1? Tell 192.168.1.1 → replies to that port"

> **A switch connects devices on the same network.** Two switches can be connected to expand the network. Routers connect different networks.

### Router

A router connects **different networks** (e.g., your LAN and the internet). It operates at the network layer (layer 3) and uses **IP addresses** to decide where to send packets.

    # What a router does:
    # Packet arrives from 192.168.1.100 destined for 93.184.216.34
    # Router checks its routing table: "93.184.216.34 → send via WAN port"
    # Before forwarding: strips the source MAC, adds its own MAC as source MAC
    # (because the previous MAC is only valid on the local segment)

    # This is why your app sees the server's public IP as the source IP in logs
    # not your machine's local IP

> **Your app usually sees the router's public IP, not your machine's local IP.** If you need the real client IP behind a proxy/CDN, use `X-Forwarded-For` header parsing. The first IP in the chain is usually the real client IP.

### NAT (Network Address Translation)

NAT maps **private IP + port** to **public IP + port**. This solves two problems:

    # Problem 1: IPv4 exhaustion (not enough public IPs for every device)
    # Problem 2: private IPs aren't routable on the internet

    # What NAT does:
    # Internal: 192.168.1.10:51234 → External: 93.184.216.34:51234
    # Your app connects to example.com:443 from 192.168.1.10
    # Router translates source IP: 192.168.1.10 → 93.184.216.34
    # When response comes back, router translates dest IP back: 93.184.216.34 → 192.168.1.10
    # Your app thinks it's talking to 192.168.1.10 but it's actually going through the router
    # with the public IP

> **Port forwarding is manual NAT for incoming connections.** Outgoing connections are NAT'd automatically. But if you run a web server on 192.168.1.10:8000, the internet can't reach it because no incoming NAT mapping exists yet. You must explicitly configure: "forward port 443 to 192.168.1.10:8000" on the router.

### Firewall

A firewall controls **which packets are allowed through** based on rules: source/dest IP, port, protocol, connection state (new/established/related).

    # Types:
    # - Network firewall: on your router (controls traffic between LAN and internet)
    # - Host firewall: on your machine (controls traffic to/from your machine)
    # - Cloud security groups: on your cloud provider (controls traffic to/from VMs)

    # How rules are evaluated (usually, in order):
    # 1. Allow established connections (return traffic for existing connections)
    # 2. Block specific things (drop packets matching deny rules)
    # 3. Allow everything else (default allow or default deny)

    # Common patterns:
    # Allow all outgoing (default on most setups)
    # Allow incoming: only specific ports (SSH=22, HTTP=80, HTTPS=443)
    # Deny all incoming by default
    # Allow ping (ICMP) for diagnostics
    # Block specific IPs (e.g., known bad actors)

> **"Connection timed out" with nothing in logs = silently dropped by firewall.** Firewalls often drop packets without logging. Check both the cloud security group AND the server's host firewall. Try `curl -v` on the server itself to see if the port is actually listening. If yes, it's the firewall. If "Connection refused," it's not.

### VPN (Virtual Private Network)

A VPN creates an encrypted tunnel through the internet to a private network. What this actually means:

    # Without VPN:
    # Your laptop → open internet → raw packets → server on 203.0.113.1 → MySQL on 3306
    # Anyone between you and the server can see you're connecting to 203.0.113.1 on port 3306
    # They don't see what's inside the packets (TLS helps, but the metadata is visible)

    # With VPN:
    # Your laptop → VPN client → encrypted tunnel → VPN server → private network → MySQL on 10.0.0.5:3306
    # Outsiders only see: you're sending encrypted data to 203.0.113.1
    # They cannot see: the destination (10.0.0.5) or the port (3306)
    # It looks like you're on the private network

> **A VPN does NOT encrypt your app's actual data.** It encrypts the transport. The app inside the VPN tunnel is still HTTP/JSON/<whatever> — it's just delivered through an encrypted tunnel. You still need HTTPS for application-level encryption.

### Subnets

    # A subnet divides one IP range into smaller networks
    # 192.168.1.0/24 means:
    #   Network: 192.168.1.0
    #   First usable: 192.168.1.1
    #   Last usable: 192.168.1.254
    #   Broadcast: 192.168.1.255
    #   Total usable IPs: 254
    # /24 = 255 addresses (256 total, 1 network, 254 usable)
    # /25 = 128 addresses (126 usable)
    # /26 = 64 addresses (62 usable)
    # /27 = 32 addresses (30 usable)
    # /28 = 16 addresses (14 usable)
    # /32 = 1 address (1 usable, commonly used for single hosts)

    # Subnet mask tells the OS which part of the IP is "network" and which is "host"
    # 255.255.255.0 → /24 → last octet is host (254 usable hosts)
    # 255.255.255.128 → /25 → last 7 bits are host (126 usable hosts)
    # 255.255.255.192 → /26 → last 6 bits are host (62 usable hosts)

    # Default gateway: the router's IP in the subnet (usually .1)
    # DNS server: usually the router's IP (same as default gateway)
    # Subnet vs broadcast: don't use .0 (network) or .255 (broadcast) as host addresses
    # Those have special meaning in the protocol

### Load Balancer

A load balancer distributes incoming requests across multiple backend servers. What it actually does:

    # Client → load balancer → picks backend
    # Load balancer → backend-1 (forwards request, remembers which client went where)
    # Same client → same backend (session affinity / sticky sessions)
    # Backend-1 goes down → health check fails → stop sending traffic to it
    # Load balancer → routes around it to healthy backends

    # Common algorithms:
    # Round-robin: 1→1→1→1→1 (fair distribution)
    # Least connections: to the backend with fewest active connections
    # IP hash: same client → same backend (sticky sessions)
    # Weighted round-robin: weighted 3:1 means backend-1 gets 3 requests for every 1 to backend-2
    # Random: random backend each time (good for APIs with caching)

    # Health checks (how the balancer knows if a backend is alive):
    # GET /health → 200 OK → healthy
    # GET /health → 500 → unhealthy
    # Run on every 5-10 seconds per backend

### Proxy

    # Forward proxy: client → proxy → server (client doesn't know about the real server)
    # Example: Nginx sitting in front of your app, forwarding to localhost:8000
    # Client connects to proxy's port, proxy connects to app

    # Reverse proxy: client → reverse proxy → backend (client doesn't know about backends)
    # Example: Nginx receives all traffic for example.com and routes it to different backends based on path:
    #   / → app-server:8000
    #   /api → api-server:8001
    #   /ws → websocket-server:8002

    # A reverse proxy also handles:
    # - SSL termination (proxy handles TLS, backend sees plain HTTP internally)
    # - Buffering (proxy buffers responses, not the app)
    # - Compression (proxy compresses responses)
    # - Rate limiting (proxy rejects excessive requests)
    # - Static file serving (proxy serves images/CSS/JS directly, never hits your app)

> **Nginx as reverse proxy is extremely common.** Almost every production deployment has one. It handles SSL, static files, compression, and routing. Your app only ever receives requests that need dynamic content.

### CDN (Content Delivery Network)

    # What it does: cache your static files at edge locations worldwide
    # User in Tokyo requests image.jpg → CDN edge server in Tokyo serves it locally (20ms instead of 200ms from your server)

    # What it caches: static files (images, CSS, JS, fonts, videos, PDFs)
    # What it doesn't cache: dynamic responses (API calls, personalized pages, authenticated pages)
    # (unless configured otherwise)

    # Common: Cloudflare, Fastly, AWS CloudFront, Vercel, Netlify

    # CDN → checks cache → cache hit: served immediately
    # CDN → cache miss → fetches from origin → stores and serves

    # Cache invalidation:
    # Manual: "Purge all" or "Purge by URL" in CDN dashboard
    # Cache-Control header: "no-cache" or "max-age=0" tells CDN not to cache
    # Versioned files: style.css?v=2 → changing the ?v=3 invalidates the old version
    # (this is why you see ?v=hash on CSS/JS files in browsers)