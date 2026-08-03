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
for cmd in cargo jq awk find grep; do
    command -v "$cmd" >/dev/null 2>&1 || { printf 'Error: %s not found.\n' "$cmd" >&2; exit 1; }
done

# ---------- メタデータ取得 ----------
METADATA_JSON=$(cargo metadata --no-deps --format-version 1)
[ -z "$METADATA_JSON" ] && { printf 'Error: Failed to obtain metadata.\n' >&2; exit 1; }
WORKSPACE_ROOT=$(echo "$METADATA_JSON" | jq -r '.workspace_root')

# ---------- 更新関数 ----------
# update_file <path> <type:toml|json> <version>
# Updated files are appended to $TOUCHED_LIST so the post-update assertion
# (see below) knows exactly what to re-check, dry-run included.
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
    [ -n "$TOUCHED_LIST" ] && printf '%s\n' "$file_path" >> "$TOUCHED_LIST"
}

# fix_workspace_self_deps <root_cargo_toml> <version>
#
# version.sh's own [package] version = "..." match (above) does not touch
# [workspace.dependencies] entries, because a line like
#   mdka = { version = "2.1.6", path = "." }
# begins with the crate name, not "version". This has drifted silently across
# three prior releases (2.1.3-2.1.5, caught only by hand at 2.1.6 and 2.1.7).
#
# Scoped deliberately to *self-referencing* workspace-dependency entries only
# (path = "." or path pointing at a workspace member) so an external pinned
# dependency's version is never touched by accident - this only rewrites a
# workspace member's reference to itself.
fix_workspace_self_deps() {
    root_toml=$1; ver=$2
    [ ! -f "$root_toml" ] && return

    if [ "$DRY_RUN" -eq 1 ]; then
        if grep -qE '^\[workspace\.dependencies\]' "$root_toml"; then
            printf '  (dry-run) would check [workspace.dependencies] self-references in %s\n' "$root_toml"
        fi
        return
    fi

    tmp=$(mktemp) || exit 1
    awk -v nv="$ver" '
        /^\[workspace\.dependencies\]/ { in_wsdeps=1; print; next }
        /^\[/ && !/^\[workspace\.dependencies\]/ { in_wsdeps=0; print; next }
        in_wsdeps && /path[[:space:]]*=[[:space:]]*"\./ {
            if (match($0, /version[[:space:]]*=[[:space:]]*"[^"]*"/)) {
                before = substr($0, 1, RSTART - 1)
                after  = substr($0, RSTART + RLENGTH)
                print before "version = \"" nv "\"" after
                next
            }
        }
        { print }
    ' "$root_toml" > "$tmp"

    if ! diff -q "$root_toml" "$tmp" >/dev/null 2>&1; then
        mv "$tmp" "$root_toml"
        git add "$root_toml"
        printf '  updated [workspace.dependencies] self-references in %s\n' "$root_toml"
        [ -n "$TOUCHED_LIST" ] && printf '%s\n' "$root_toml" >> "$TOUCHED_LIST"
    else
        rm -f "$tmp"
    fi
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

    # Previous version, captured before anything is written, for the
    # post-update assertion below. All workspace members are expected to be
    # in lockstep, so any one package's current version serves as "the
    # previous version" that must not survive the bump anywhere.
    OLD_VERSION=$(echo "$METADATA_JSON" | jq -r '.packages[0].version')

    TOUCHED_LIST=$(mktemp) || exit 1

    printf 'Starting update to version "%s" (from "%s")...\n' "$NEW_VERSION" "$OLD_VERSION"

    # cargo metadata から各クレートのパスを抽出
    echo "$METADATA_JSON" | jq -r '.packages[] | .manifest_path' | while read -r cargo_toml; do
        crate_dir=$(dirname "$cargo_toml")

        # 1. Cargo.toml 更新
        update_file "$cargo_toml" "toml" "$NEW_VERSION"

        # 2. [拡張] 直下のサブディレクトリにある package.json を検索・更新
        # find で crate_dir の直下 (-maxdepth 1) のディレクトリを探し、
        # その中にある package.json を見つける
        find "$crate_dir" -mindepth 1 -maxdepth 2 -type d -print0 2>/dev/null | \
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

    # 4. [workspace.dependencies] の自己参照エントリを更新
    #    (see fix_workspace_self_deps comment above)
    fix_workspace_self_deps "$WORKSPACE_ROOT/Cargo.toml" "$NEW_VERSION"

    # Cargo.lock の更新（dry-run でない場合のみ）
    if [ "$DRY_RUN" -eq 0 ]; then
        cargo fetch >/dev/null 2>&1
        [ -f "Cargo.lock" ] && git add Cargo.lock
    fi

    # 5. 更新後アサーション: 触ったファイルのどれかに旧バージョン文字列が
    #    まだ残っていないか確認する。中途半端な更新は静かに成功してはならない。
    #
    # This is the more important half of Slice 3 (RFC 015). Fixing the
    # workspace.dependencies line above only fixes the one location known to
    # have drifted; this assertion catches *any* touched manifest that still
    # carries the previous version string, including locations added later
    # that nobody remembered to teach this script about.
    if [ "$DRY_RUN" -eq 0 ]; then
        sort -u "$TOUCHED_LIST" | while read -r touched_file; do
            [ -f "$touched_file" ] || continue
            if grep -qF "\"$OLD_VERSION\"" "$touched_file"; then
                printf 'STALE VERSION: %s still contains "%s"\n' "$touched_file" "$OLD_VERSION" >&2
                echo "stale" >> "$TOUCHED_LIST.stale"
            fi
        done
        if [ -f "$TOUCHED_LIST.stale" ]; then
            printf 'Error: version bump to "%s" is incomplete - at least one manifest still contains "%s". Fix the file(s) listed above before proceeding; do not commit a half-applied bump.\n' "$NEW_VERSION" "$OLD_VERSION" >&2
            rm -f "$TOUCHED_LIST" "$TOUCHED_LIST.stale"
            exit 1
        fi
    fi
    rm -f "$TOUCHED_LIST" "$TOUCHED_LIST.stale" 2>/dev/null

    if [ "$DRY_RUN" -eq 1 ]; then
        printf 'Dry run complete. Nothing was written or verified.\n'
    else
        printf 'Version update to "%s" complete and verified: no manifest retains "%s".\n' "$NEW_VERSION" "$OLD_VERSION"
    fi
fi
