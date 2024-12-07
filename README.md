# ramen

An easier way to define and parse arguments in SHELL scripts.

Usage

```bash
#!/usr/bin/env bash

set -euo pipefail

program='
version: 1.0
program: upload
args: [SRC, DST, -v/--verbose, -t/--threads, --protocol]
'

program='
---
version: "1.0"
program: upload
args:
  - name: SRC
    action: append
  - name: DST
  - name: verbose
    short: -v
    long: --verbose
    type: boolean
  - name: threads
    short: -t
    long: --threads
    type: number
    default: 8
  - name: protocol
    short: -p
    long: --protocol
    type: string
    default: scp
    select: [scp, rsync, aws]
'
eval "$( ramen "$program" -- "$@" )"

main() {
    echo "
$SRC: $SRC
DST: $DST
verbose: $verbose
threads: $threads
protocol: $protocol
"
}

main "$@"
```