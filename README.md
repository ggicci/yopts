# ramen 🍜

An easier way to define and parse arguments in SHELL scripts. [Why ramen?](#faq)

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
  eval "$( ramen "$ARGUMENT_PARSER" -- "$@" )"

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
output_prefix: ramen_
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

### Why `ramen`? Not `getopt` or `getopts`?

I’ve never been a fan of `getopt` or `getopts`. That’s why I created `ramen`. Despite spending countless hours reading their documentation and following community examples, I always came away empty-handed, unable to retain anything. For me, learning either of the two just isn’t worth the effort.

`ramen` takes a different approach by allowing you to define the argument parser in a descriptive YAML format. This simplifies the syntax while leveraging the powerful parsing capabilities of [clap](https://docs.rs/clap/latest/clap/index.html).

I know ramen is still in its early stages, but I hope it can save us time and effort when implementing argument parsers for SHELL scripts.

### Why `getopts`, not `ramen`?

`getopts` comes along with the Linux system, `ramen` doesn't.

