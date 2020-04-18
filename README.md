# Fix Compaudit Issues

A binary that takes a really long way around to run the below script but
in rust.

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

Depends on having `chmod`, `chown`, `zsh` and `compaudit` available.

## Usage

``` bash
cargo run
```

``` bash
./fix-compaudit
```

## License

[CC0](LICENSE.md) - Public Domain
