etcddata
===
## run
etcddata dump -h 127.0.0.1 -p 2379 -d db

## build on Arch Linux
``` bash
export OPENSSL_LIB_DIR="/usr/lib/openssl-1.0"
export OPENSSL_INCLUDE_DIR="/usr/include/openssl-1.0"
cargo build
```