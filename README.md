# DDNS

Dynamic DNS for multiple DNS providers

## Provider
The following DNS providers are currently supported:

* AWS Route53

## Usage

```
ddns 0.1.0
Multi-provider dynamic DNS

USAGE:
    ddns [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -o, --once       Run ddns once and exit
    -V, --version    Prints version information

OPTIONS:
    -c, --config-file <config-file>    Path to the config file [env: CONFIG_FILE=]  [default: ddns.toml]
    -l, --log-level <log-level>        Enables different levels of log messages [env: LOG_LEVEL=]  [default: info]
```

## Configuration

### AWS
`ddns` relies on the standard AWS configuration strategies, (shared credentials, `AWS_ACCESS_KEY`, `AWS_SECRET_KEY`, Instance Profile, etc).

### Config file
Configuration is stored in a `toml` file. The default path for the config file is `./ddns.toml` but this can be altered using the `DDNS_CONFIG_FILE` environment variable.

```
[global]
interval_seconds = 300

[[entries]]
domain = "jellyfin.mydomain.net"

    [entries.provider]
    type = "aws"
    hosted_zone_id = "ABCDEFG1234567"

[[entries]]
domain = "matrix.mydomain.com"

    [entries.provider]
    type = "aws"
    hosted_zone_id = "1234567ABCDEFG"
```
