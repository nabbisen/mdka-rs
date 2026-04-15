#!/bin/sh
#
# version.sh – Cargo ワークスペースのクレートバージョンを
#   * 一覧表示 (--list)
#   * 任意のバージョンに一括更新 (--update <ver>)
#
# 必要ツール: cargo, jq, awk, grep
#
# 使い方例:
#   sh version.sh --list
#   sh version.sh -u 1.2.3
#   sh version.sh --update 1.2.3 --dry-run
#

# ---------- 定数 ----------
CARGO_LOCK=./Cargo.lock

# ---------- ヘルプ ----------
show_help() {
    cat <<EOF
Usage: ${0##*/} [OPTIONS]

Options:
  -l, --list                List each crate with its current version.
  -u, --update VERSION      Set all crates to VERSION.
  -d, --dry-run             Show what would be changed, but do not modify files.
  -h, --help                Show this help and exit.

If no option is given, this help text is shown.

Examples:
  ${0##*/} --list
  ${0##*/} --update 1.2.3
  ${0##*/} --update 1.2.3 --dry-run
EOF
    exit 0
}

# ---------- 引数解析 ----------
LIST_MODE=0
UPDATE_MODE=0
DRY_RUN=0
NEW_VERSION=

# オプションが全く無いときはヘルプを出すフラグ
NO_OPTION=1

while [ $# -gt 0 ]; do
    case "$1" in
        -l|--list)          LIST_MODE=1; NO_OPTION=0; shift ;;
        -u|--update)        UPDATE_MODE=1; NO_OPTION=0; shift
                             if [ $# -eq 0 ]; then
                                 printf 'Error: --update requires a version argument.\n' >&2
                                 exit 1
                             fi
                             NEW_VERSION=$1; shift ;;
        -d|--dry-run)       DRY_RUN=1; NO_OPTION=0; shift ;;
        -h|--help)          show_help ;;
        *) printf 'Unknown option: %s\n' "$1" >&2; exit 1 ;;
    esac
done

# 何もオプションが無ければヘルプを表示
[ "$NO_OPTION" -eq 1 ] && show_help

# ---------- 必要コマンドの存在確認 ----------
command -v cargo >/dev/null 2>&1 || { printf 'Error: cargo not found in PATH.\n' >&2; exit 1; }
command -v jq >/dev/null 2>&1   || { printf 'Error: jq not found in PATH.\n' >&2; exit 1; }
command -v awk >/dev/null 2>&1  || { printf 'Error: awk not found in PATH.\n' >&2; exit 1; }

# ---------- Cargo metadata 取得 ----------
# JSON の配列: [{name, version, manifest_path}, …]
METADATA_JSON=$(cargo metadata --no-deps --format-version 1)

if [ -z "$METADATA_JSON" ]; then
    printf 'Error: Failed to obtain cargo metadata.\n' >&2
    exit 1
fi

# ---------- バージョン一覧表示 ----------
if [ "$LIST_MODE" -eq 1 ]; then
    printf 'Crate versions in this workspace:\n'
    # jq で name, version, manifest_path をタブ区切りで出力
    echo "$METADATA_JSON" |
        jq -r '.packages[] | "\(.name)\t\(.version)\t\(.manifest_path)"' |
        while IFS=$'\t' read -r crate_name crate_version manifest_path; do
            printf '  %s : %s\n' "$crate_name" "$crate_version"
        done
    # list だけで終了
    [ "$UPDATE_MODE" -eq 0 ] && exit 0
fi

# ---------- バージョン一括更新 ----------
if [ "$UPDATE_MODE" -eq 1 ]; then
    if [ -z "$NEW_VERSION" ]; then
        printf 'Error: No new version supplied.\n' >&2
        exit 1
    fi

    printf 'Updating all sub directories (crates etc.) to version "%s"%s\n' \
           "$NEW_VERSION" "$( [ "$DRY_RUN" -eq 1 ] && printf ' (dry‑run)' )"

    # 再度 JSON から name, version, manifest_path を取得して処理
    echo "$METADATA_JSON" |
        jq -r '.packages[] | "\(.name)\t\(.version)\t\(.manifest_path)"' |
        while IFS=$'\t' read -r crate_name old_version manifest_path; do
            # manifest_path が実際に存在するか確認
            if [ ! -f "$manifest_path" ]; then
                printf '  %s : manifest not found (skipped)\n' "$crate_name" >&2
                continue
            fi

            if [ "$DRY_RUN" -eq 1 ]; then
                printf '  %s : %s → %s (would modify)\n' \
                       "$crate_name" "$old_version" "$NEW_VERSION"
            else
                # awk で version 行を書き換えて一時ファイルに保存、成功したら上書き
                tmp=$(mktemp) || exit 1
                awk -v nv="$NEW_VERSION" '
                    $0 ~ /^[[:space:]]*version[[:space:]]*=/ {
                        print "version = \"" nv "\""
                        next
                    }
                    { print }
                ' "$manifest_path" >"$tmp" && mv "$tmp" "$manifest_path"

                printf '  %s : %s → %s (updated)\n' \
                       "$crate_name" "$old_version" "$NEW_VERSION"

                git add "$manifest_path"
            fi
        done
        if [ -f "$CARGO_LOCK" ]; then
            sleep 5
            git add "$CARGO_LOCK"
        else
            printf 'Warn: %s is missing. failed to git add.\n' "$CARGO_LOCK" >&2
        fi
fi

exit 0
