#!/bin/sh

VERSION="$1"

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  exit 1
fi

update_package_json() {
  FILE="$1"
  echo "Updating $FILE to version $VERSION"

  tmpfile="$(mktemp)"

  jq --arg ver "$VERSION" '
    .version = $ver
    | if .optionalDependencies != null then
        .optionalDependencies |= with_entries(.value = $ver)
      else .
      end
  ' "$FILE" > "$tmpfile" && mv "$tmpfile" "$FILE"

  git add "$FILE"
}

# (2) カレントディレクトリの package.json
if [ -f package.json ]; then
  update_package_json package.json
fi

# (3) サブディレクトリの package.json をループ
for dir in */; do
  [ -f "$dir/package.json" ] && update_package_json "$dir/package.json"
done

# (4) Cargo.toml を同期
CARGO_TOML=../Cargo.toml
CARGO_LOCK=../Cargo.lock

update_cargo_toml() {
  _CARGO_TOML="$1"
  _CARGO_LOCK="$2"

  echo "Updating $_CARGO_TOML to version $VERSION"

  # version = "..." を書き換える（[package] セクション内に限る）
  awk -v ver="$VERSION" '
    BEGIN { in_package = 0 }
    /^\[package\]/ { in_package = 1; print; next }
    /^\[/ && !/^\[package\]/ { in_package = 0; print; next }
    in_package && /^version[ \t]*=/ {
      print "version = \"" ver "\""
      next
    }
    { print }
  ' "$_CARGO_TOML" > "$_CARGO_TOML.tmp" && mv "$_CARGO_TOML.tmp" "$_CARGO_TOML"

  git add "$_CARGO_TOML" "$_CARGO_LOCK"
}

if [ -f $CARGO_TOML ]; then
  update_cargo_toml "$CARGO_TOML" "$CARGO_LOCK"
fi
