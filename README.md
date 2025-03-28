# yopts 🍜

An easier way to define and parse arguments in SHELL scripts. [Why yopts?](#faq)

_Enjoy your SHELL scripting!_

## Usage

```bash
#!/usr/bin/env bash

set -euo pipefail

ARGUMENT_PARSER='
version: "1.0"
program: upload
args: [SRC, DST, -v/--verbose, -t/--threads, --protocol]
'

main() {
  eval "$( yopts "$ARGUMENT_PARSER" -- "$@" )"

    echo "
SRC: $SRC
DST: $DST
verbose: $verbose
threads: $threads
protocol: $protocol
"
}

main "$@"
```

More granual control over arguments:

```bash
ARGUMENT_PARSER='
version: "1.0"
program: upload
output_prefix: yopts_
args:
  - name: SRC
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
```

## FAQ

### Why `yopts`? Not `getopt` or `getopts`?

I’ve never been a fan of `getopt` or `getopts`. That’s why I created `yopts`. Despite spending countless hours reading their documentation and following community examples, I always came away empty-handed, unable to retain anything. For me, learning either of the two just isn’t worth the effort.

`yopts` takes a different approach by allowing you to define the argument parser in a descriptive YAML format. This simplifies the syntax while leveraging the powerful parsing capabilities of [clap](https://docs.rs/clap/latest/clap/index.html).

I know yopts is still in its early stages, but I hope it can save us time and effort when implementing argument parsers for SHELL scripts.

### Why `getopts`, not `yopts`?

`getopts` comes along with the Linux system, `yopts` doesn't.

