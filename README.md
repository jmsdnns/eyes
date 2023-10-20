# Eyes

![Googley eyes on an otherwise very serious Samuel L Jackson, which should deflate any notion of this being an important port scanner](the_eyes.gif)

_Eyes is a nonblocking port scanner, written to help me learn Rust_ 🦀


## Using It

```shell
$ git clone https://github.com/jmsdnns/eyes
$ cd eyes
$ cargo run -- 127.0.0.1
```

## How It Works

More elaborate usage looks like this:

```shell
$ cargo run -- 127.0.0.1 -p 22,80,8000-8099
8080: open
[eyes] Finished scan
```

### PORTS

The `-p` flag is for specifying which ports to scan. Ports can be expressed as:

* list of numbers: `eyes <target> -p 22,80,1336`
* range of numbers: `eyes <target> -p 22-80`
* mix of both: `eyes <target> -p 22,80,8000-8099,443,8443,3000-3443`

### CONCURRENCY

The `-c` flag controls how many sockets are open for scanning at the same time, eg. concurrently. One coroutine is used for each port scanned.

This flag is set to `1000` by default, which is safely below the default number of open files allowed on computers, but go wild if that's your thing too.

### TIMEOUT

The `-t` flag controls how long to wait on a connection that isn't opening before decided it's just not reachable.

The default timeout is 3 seconds.

### HELP

```shell
$ cargo run -- -h

Usage: eyes [OPTIONS] <target>

Arguments:
  <target>  The IP to scan

Options:
  -v, --verbose                    Display detailed information
  -p, --ports <ports>              List of ports to scan [default: 1-1024]
  -c, --concurrency <concurrency>  Number of simultaneous scanners [default: 1000]
  -t, --timeout <timeout>          Connection timeout [default: 3]
  -h, --help                       Print help
```

#### hack the planet
