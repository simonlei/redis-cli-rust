# Redis command line client for rust

A redis command line client for rust, just like redis-cli

## Current support

### Command line args

```-h host -p port -a pasword```

### Redis commands

All redis command will send to redis server, and output to console.

### Other feature

1. Support pipeline, such as ```# echo 'get abc' | redis-cli-rust```
2. Support display CJK directly instead of unicode, for example display 中文 instead of
   \xe4\xb8\xad\xe6\x96\x87

## Not support yet

1. No hints for input
2. Other command line args