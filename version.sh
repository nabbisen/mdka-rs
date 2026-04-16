#!/bin/sh
#
# version.sh – Cargo, Node.js, Python 関連ファイルのバージョンを一括更新
#
# 必要ツール: cargo, jq, awk, grep, find

# ---------- ヘルプ ----------
show_help() {
    cat <<EOF
Usage: ${0##*/} [OPTIONS]

Options:
  -l, --list                List each crate with its current version.
  -u, --update VERSION      Set all Cargo, npm, and pip files to VERSION.
                            Includes package.json in subdirectories of packages.
  -d, --dry-run             Show what would be changed, but do not modify files.
  -h, --help                Show this help and exit.

Examples:
  ${0##*/} --list
  ${0##*/} --update 1.2.3
EOF
    exit 0
}

# ---------- 引数解析 ----------
LIST_MODE=0; UPDATE_MODE=0; DRY_RUN=0; NEW_VERSION=; NO_OPTION=1

while [ $# -gt 0 ]; do
    case "$1" in
        -l|--list)    LIST_MODE=1; NO_OPTION=0; shift ;;
        -u|--update)  UPDATE_MODE=1; NO_OPTION=0; NEW_VERSION=$2; shift 2 ;;
        -d|--dry-run) DRY_RUN=1; NO_OPTION=0; shift ;;
        -h|--help)    show_help ;;
        *) printf 'Unknown option: %s\n' "$1" >&2; exit 1 ;;
    esac
done
[ "$NO_OPTION" -eq 1 ] && show_help

# ---------- ツール確認 ----------
for cmd in cargo jq awk find; do
    command -v "$cmd" >/dev/null 2>&1 || { printf 'Error: %s not found.\n' "$cmd" >&2; exit 1; }
done

# ---------- メタデータ取得 ----------
METADATA_JSON=$(cargo metadata --no-deps --format-version 1)
[ -z "$METADATA_JSON" ] && { printf 'Error: Failed to obtain metadata.\n' >&2; exit 1; }

# ---------- 更新関数 ----------
# update_file <path> <type:toml|json> <version>
update_file() {
    file_path=$1; type=$2; ver=$3
    [ ! -f "$file_path" ] && return

    if [ "$DRY_RUN" -eq 1 ]; then
        printf '  (dry-run) would update %s\n' "$file_path"
        return
    fi

    tmp=$(mktemp) || exit 1
    if [ "$type" = "toml" ]; then
        # TOML用: 最初の [package] や [project] セクション直後の version を狙い撃ち
        awk -v nv="$ver" '
            !found && /^[[:space:]]*version[[:space:]]*=/ {
                print "version = \"" nv "\""
                found=1; next
            }
            { print }
        ' "$file_path" > "$tmp"
    else
        # JSON用: jq で確実に更新
        jq --arg v "$ver" '.version = $v' "$file_path" > "$tmp"
    fi

    mv "$tmp" "$file_path"
    git add "$file_path"
    printf '  updated %s\n' "$file_path"
}

# ---------- メイン処理 ----------

# 1. バージョン一覧表示
if [ "$LIST_MODE" -eq 1 ]; then
    printf 'Current versions:\n'
    echo "$METADATA_JSON" | jq -r '.packages[] | "\(.name)\t\(.version)"' | \
        awk -F'\t' '{ printf "  %-20s : %s\n", $1, $2 }'
    [ "$UPDATE_MODE" -eq 0 ] && exit 0
fi

# 2. バージョン更新
if [ "$UPDATE_MODE" -eq 1 ]; then
    [ -z "$NEW_VERSION" ] && { printf 'Error: Missing version.\n' >&2; exit 1; }

    printf 'Starting update to version "%s"...\n' "$NEW_VERSION"

    # cargo metadata から各クレートのパスを抽出
    echo "$METADATA_JSON" | jq -r '.packages[] | .manifest_path' | while read -r cargo_toml; do
        crate_dir=$(dirname "$cargo_toml")
        
        # 1. Cargo.toml 更新
        update_file "$cargo_toml" "toml" "$NEW_VERSION"

        # 2. [拡張] 直下のサブディレクトリにある package.json を検索・更新
        # find で crate_dir の直下 (-maxdepth 1) のディレクトリを探し、
        # その中にある package.json を見つける
        find "$crate_dir" -mindepth 1 -maxdepth 1 -type d -print0 2>/dev/null | \
        while IFS= read -r -d '' subdir; do
            sub_pkg_json="$subdir/package.json"
            if [ -f "$sub_pkg_json" ]; then
                update_file "$sub_pkg_json" "json" "$NEW_VERSION"
            fi
            sub_pkg_lock_json="$subdir/package-lock.json"
            if [ -f "$sub_pkg_lock_json" ]; then
                update_file "$sub_pkg_lock_json" "json" "$NEW_VERSION"
            fi
        done

        # 3. 同一ディレクトリ内の pyproject.toml をチェック
        update_file "$crate_dir/pyproject.toml" "toml" "$NEW_VERSION"
    done

    # Cargo.lock の更新（dry-run でない場合のみ）
    if [ "$DRY_RUN" -eq 0 ]; then
        cargo fetch >/dev/null 2>&1
        [ -f "Cargo.lock" ] && git add Cargo.lock
    fi
fi