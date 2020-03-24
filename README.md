# addns

Dynamic DNS for an AWS route 53 record

## Configuration

### AWS
addns relies on the standard AWS configuration strategies, (`aws configure`, `AWS_ACCESS_KEY`, `AWS_SECRET_KEY`, etc).

### Config file
Configuration is stored in a `toml` file. Currently only AWS Route53 is supported as a DNS provider.

```
domain = "srv01.mydomain.com"
provider = "aws"
interval_seconds = 60 # Optional (default 3600)

# This block is required when provider is set to "aws"
[aws]
hosted_zone_id = "ABCDEFG1234567"
ttl = 300 # Optional (default 300)
```
