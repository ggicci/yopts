# ramen

An easier way to define and parse arguments in SHELL scripts.

Usage

```bash
#!/usr/bin/env bash

set -euo pipefail

program="
---
program: upload
positionals: SRC DST
args:
  - name: verbose
    type: boolean
    required: false
  - name: threads
    type: number
    required: false
    default: 8
  - name: protocol
    type: string
    required: false
    default: scp
    select: [scp, rsync, aws]
"
eval "$( ramen "$program" )"

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