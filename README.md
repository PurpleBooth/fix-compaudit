# Fix Compaudit Issues

A binary that takes a really long way around to run the below script but
in rust.

Depends on having `chmod`, `chown`, `zsh` and `compaudit` available.

## Usage

``` bash
fix-compaudit
```

## Installing

First tap my homebrew repo

``` shell
brew tap PurpleBooth/repo
```

Next install the binary

``` shell
brew install fix-compaudit
```

You can also download the [latest
release](https://github.com/PurpleBooth/alfred-emoji-snippet-pack/releases/latest)
and run it.

## How does it work

Well here's a version of this in bash, if that makes it clearer

``` shell
#!/usr/bin/env bash

set -euo pipefail

mapfile -t AUDIT_PROBLEMS < <(zsh -c "autoload -U compaudit && compaudit" 2>/dev/null)

if [ "${#AUDIT_PROBLEMS[@]}" -eq 0 ]; then
    exit 0
fi

for FILE in "${AUDIT_PROBLEMS[@]}"; do
    if [ "$(gstat -c '%U' "$FILE")" != "$USER" ]; then
        echo "gaining ownership: $FILE"
        sudo chown -R "$USER" "$FILE"
    fi
    if [ "$(gstat -c '%U' "$FILE")" != "$USER" ]; then
        echo "gaining ownership: $FILE"
        sudo chown -R "$USER" "$FILE"
    fi

    if gstat -c "%A" "$FILE" | grep -qE '(.{2}w.{7}|.{5}w.{4})'; then
        echo "removing non-user write: $FILE"
        chmod -R go-w "$FILE"
    fi
    if gstat -c "%A" "$FILE" | grep -qE '(.{2}w.{7}|.{5}w.{4})'; then
        echo "removing non-user write: $FILE"
        chmod -R go-w "$FILE"
    fi
done
```

## License

[CC0](LICENSE.md) - Public Domain
