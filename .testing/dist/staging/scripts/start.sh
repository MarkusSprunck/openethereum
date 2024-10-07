#!/bin/sh

/home/openethereum/openethereum --config /home/parity/authority.toml 2>&1 | /bin/sh /home/openethereum/format2json.sh