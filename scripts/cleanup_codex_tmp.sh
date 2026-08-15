#!/usr/bin/env bash
#
# Audit and, only with --delete-safe, remove disposable Codex workspaces that
# are demonstrably related to this checkout.  The default is deliberately a
# dry run.  This script is intentionally conservative: an inconclusive check
# retains the directory.

set -u
set -o pipefail

MODE=dry-run
TMP_ROOT=${CODEX_TMP_ROOT:-/private/tmp}

usage() {
    cat <<'EOF'
Usage: scripts/cleanup_codex_tmp.sh [--delete-safe] [--tmp-root PATH]

Without --delete-safe this is an audit only.  --delete-safe removes only:
  * clean registered worktrees whose reachable history is on origin;
  * clean independent clones with the authoritative origin (or a local clone
    of this authoritative checkout) whose history is on origin; and
  * standalone Cargo target directories that identify this package through a
    Cargo fingerprint and contain only standard target-root entries.

CODEX_TMP_ROOT is provided for isolated testing.  Production use defaults to
/private/tmp.
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --delete-safe)
            MODE=delete-safe
            ;;
        --tmp-root)
            shift
            [ "$#" -gt 0 ] || { usage >&2; exit 2; }
            TMP_ROOT=$1
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            usage >&2
            exit 2
            ;;
    esac
    shift
done

say() {
    printf '%s\n' "$*"
}

fail() {
    say "NO-GO: $*" >&2
    exit 1
}

canonical_dir() {
    (cd -P -- "$1" 2>/dev/null && pwd -P)
}

size_kib() {
    du -sk -- "$1" 2>/dev/null | awk '{print $1}'
}

trim_count() {
    tr -d '[:space:]'
}

is_below_tmp_root() {
    case "$1" in
        "$TMP_ROOT"/*) return 0 ;;
        *) return 1 ;;
    esac
}

if ! command -v git >/dev/null 2>&1; then
    fail "git is required"
fi

AUTH_ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || fail "run inside the authoritative Git checkout"
AUTH_ROOT=$(canonical_dir "$AUTH_ROOT") || fail "cannot canonicalize authoritative checkout"
AUTH_COMMON_RAW=$(git rev-parse --git-common-dir 2>/dev/null) || fail "cannot determine Git common directory"
case "$AUTH_COMMON_RAW" in
    /*) AUTH_COMMON=$(canonical_dir "$AUTH_COMMON_RAW") ;;
    *) AUTH_COMMON=$(canonical_dir "$AUTH_ROOT/$AUTH_COMMON_RAW") ;;
esac
[ -n "$AUTH_COMMON" ] || fail "cannot canonicalize Git common directory"
AUTH_ORIGIN=$(git remote get-url origin 2>/dev/null) || fail "the authoritative checkout has no origin remote"
[ -n "$AUTH_ORIGIN" ] || fail "the authoritative origin URL is empty"
AUTH_PACKAGE=$(awk '
    /^\[package\]$/ { in_package=1; next }
    /^\[/ { in_package=0 }
    in_package && $1 == "name" { gsub(/\"/, "", $3); print $3; exit }
' "$AUTH_ROOT/Cargo.toml" 2>/dev/null || true)

[ -d "$TMP_ROOT" ] || fail "temporary root does not exist: $TMP_ROOT"
TMP_ROOT=$(canonical_dir "$TMP_ROOT") || fail "cannot canonicalize temporary root"
[ "$TMP_ROOT" != / ] || fail "refusing an unsafe temporary root"

REMOTE_TAGS_FILE=$(mktemp "${TMPDIR:-/tmp}/cleanup-codex-remote-tags.XXXXXX") || fail "cannot create verification cache"
trap 'rm -f -- "$REMOTE_TAGS_FILE"' EXIT HUP INT TERM

say "AUTHORITATIVE_REPOSITORY=$AUTH_ROOT"
say "CANONICAL_ORIGIN=$AUTH_ORIGIN"
say "TMP_ROOT=$TMP_ROOT"
say "MODE=$MODE"

# Fetch is a hard gate.  Local remote-tracking refs are not trusted until this
# completes, and ls-remote also lets tag-only preservation be verified.
if ! git fetch origin --prune --tags; then
    fail "origin fetch failed; no workspace was changed"
fi
if ! git ls-remote --heads --tags origin >"$REMOTE_TAGS_FILE"; then
    fail "cannot verify origin heads and tags; no workspace was changed"
fi

REMOTE_REFS=$(git for-each-ref --format='%(refname)' refs/remotes/origin)

remote_tag_matches_local_ref() {
    local tag_name=$1
    local local_oid
    local_oid=$(git rev-parse "refs/tags/$tag_name" 2>/dev/null) || return 1
    awk -v oid="$local_oid" -v ref="refs/tags/$tag_name" '$1 == oid && $2 == ref { found=1 } END { exit(found ? 0 : 1) }' "$REMOTE_TAGS_FILE"
}

# Return success only when COMMIT is reachable from a fetched origin branch or
# an origin tag whose exact tag object/ref is confirmed by ls-remote.
commit_is_preserved_on_origin() {
    local commit=$1
    local ref tag_ref tag_name

    git -C "$AUTH_ROOT" cat-file -e "${commit}^{commit}" 2>/dev/null || return 1
    while IFS= read -r ref; do
        [ -n "$ref" ] || continue
        if git -C "$AUTH_ROOT" merge-base --is-ancestor "$commit" "$ref" 2>/dev/null; then
            return 0
        fi
    done <<<"$REMOTE_REFS"

    while IFS= read -r tag_ref; do
        [ -n "$tag_ref" ] || continue
        tag_name=${tag_ref#refs/tags/}
        if remote_tag_matches_local_ref "$tag_name" && git -C "$AUTH_ROOT" merge-base --is-ancestor "$commit" "$tag_ref" 2>/dev/null; then
            return 0
        fi
    done < <(git -C "$AUTH_ROOT" for-each-ref --contains="$commit" --format='%(refname)' refs/tags)
    return 1
}

local_origin_matches_authoritative_checkout() {
    local origin=$1
    local local_origin_root
    case "$origin" in
        file://*) origin=${origin#file://} ;;
        /*) ;;
        *) return 1 ;;
    esac
    [ -d "$origin" ] || return 1
    local_origin_root=$(git -C "$origin" rev-parse --show-toplevel 2>/dev/null) || return 1
    local_origin_root=$(canonical_dir "$local_origin_root") || return 1
    [ "$local_origin_root" = "$AUTH_ROOT" ]
}

workspace_has_authoritative_package() {
    local workspace=$1
    [ -n "$AUTH_PACKAGE" ] || return 1
    [ -f "$workspace/Cargo.toml" ] || return 1
    awk -v expected="$AUTH_PACKAGE" '
        BEGIN { found=0 }
        /^\[package\]$/ { in_package=1; next }
        /^\[/ { in_package=0 }
        in_package && $1 == "name" { gsub(/\"/, "", $3); found=($3 == expected); exit }
        END { exit(found ? 0 : 1) }
    ' "$workspace/Cargo.toml"
}

safe_ignored_path() {
    case "$1" in
        target/|debug/|.venv/|venv/|.pytest_cache/|.mypy_cache/|.ruff_cache/|.cache/|\
        */target/|*/debug/|*/incremental/|*/deps/|*/build/|*/.venv/|*/venv/|\
        */__pycache__/|*/.pytest_cache/|*/.mypy_cache/|*/.ruff_cache/|*/.cache/)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

artifact_size_kib() {
    local workspace=$1
    local path total=0 amount
    while IFS= read -r -d '' path; do
        amount=$(size_kib "$path")
        amount=${amount:-0}
        total=$((total + amount))
    done < <(find "$workspace" -type d \( -name target -o -name debug -o -name incremental -o -name deps -o -name build -o -name .venv -o -name venv -o -name __pycache__ -o -name .pytest_cache -o -name .mypy_cache -o -name .ruff_cache \) -prune -print0 2>/dev/null)
    printf '%s\n' "$total"
}

is_registered_worktree() {
    local workspace=$1
    git worktree list --porcelain | awk -v candidate="$workspace" '
        /^worktree / { if (substr($0, 10) == candidate) found=1 }
        END { exit(found ? 0 : 1) }
    '
}

SAFE_COUNT=0
UNSAFE_COUNT=0
UNKNOWN_COUNT=0
DELETED_COUNT=0
STALE_PRUNE_ALLOWED=1

report_workspace() {
    local classification=$1 workspace=$2 kind=$3 origin=$4 branch=$5 head=$6 state=$7 stash_state=$8 artifact_kib=$9 reason=${10}
    printf 'WORKSPACE|%s|path=%s|type=%s|size_kib=%s|origin=%s|branch=%s|head=%s|state=%s|stashes=%s|artifact_kib=%s|action=%s\n' \
        "$classification" "$workspace" "$kind" "$(size_kib "$workspace")" "${origin:-NONE}" "$branch" "$head" "$state" "$stash_state" "$artifact_kib" "$reason"
}

add_reason() {
    if [ -z "$REASONS" ]; then
        REASONS=$1
    else
        REASONS="$REASONS; $1"
    fi
}

delete_independent_clone() {
    local workspace=$1
    is_below_tmp_root "$workspace" || return 1
    [ "$workspace" != "$TMP_ROOT" ] || return 1
    [ ! -L "$workspace" ] || return 1
    rm -rf -- "$workspace"
}

inspect_git_workspace() {
    local workspace=$1
    local common_raw common origin head branch kind relation status ignored_lines line ignored_path
    local tracked_changes untracked_files stash_count artifact_kib ref sha
    local REASONS=""

    [ -d "$workspace" ] || return 0
    [ ! -L "$workspace" ] || {
        report_workspace UNKNOWN "$workspace" "Git workspace" "" "UNKNOWN" "UNKNOWN" "symlink" "UNKNOWN" 0 "retain: symlinked workspace"
        UNKNOWN_COUNT=$((UNKNOWN_COUNT + 1))
        return 0
    }
    git -C "$workspace" rev-parse --is-inside-work-tree >/dev/null 2>&1 || return 0

    common_raw=$(git -C "$workspace" rev-parse --git-common-dir 2>/dev/null) || {
        report_workspace UNKNOWN "$workspace" "Git workspace" "" "UNKNOWN" "UNKNOWN" "Git metadata unreadable" "UNKNOWN" 0 "retain: cannot inspect Git common directory"
        UNKNOWN_COUNT=$((UNKNOWN_COUNT + 1))
        return 0
    }
    case "$common_raw" in
        /*) common=$(canonical_dir "$common_raw") ;;
        *) common=$(canonical_dir "$workspace/$common_raw") ;;
    esac
    origin=$(git -C "$workspace" remote get-url origin 2>/dev/null || true)
    head=$(git -C "$workspace" rev-parse HEAD 2>/dev/null || printf UNKNOWN)
    branch=$(git -C "$workspace" symbolic-ref --quiet --short HEAD 2>/dev/null || printf DETACHED)
    artifact_kib=$(artifact_size_kib "$workspace")
    relation=none
    kind="independent temporary clone"
    if [ "$common" = "$AUTH_COMMON" ] && is_registered_worktree "$workspace"; then
        relation=registered
        kind="registered Git worktree"
    elif [ "$origin" = "$AUTH_ORIGIN" ]; then
        relation=independent
    elif local_origin_matches_authoritative_checkout "$origin"; then
        relation=independent
        kind="independent clone of authoritative checkout"
    fi

    if [ "$relation" = none ]; then
        if workspace_has_authoritative_package "$workspace"; then
            report_workspace UNKNOWN "$workspace" "Git workspace" "$origin" "$branch" "$head" "cleanliness untrusted" "UNKNOWN" "$artifact_kib" "retain: package matches but repository relationship is unproven"
            UNKNOWN_COUNT=$((UNKNOWN_COUNT + 1))
        else
            report_workspace UNRELATED "$workspace" "Git workspace" "$origin" "$branch" "$head" "not inspected further" "UNKNOWN" "$artifact_kib" "untouched: origin/common directory do not prove relation"
        fi
        return 0
    fi

    status=$(git -C "$workspace" status --porcelain=v1 --untracked-files=all 2>/dev/null) || {
        report_workspace UNKNOWN "$workspace" "$kind" "$origin" "$branch" "$head" "status failed" "UNKNOWN" "$artifact_kib" "retain: cannot inspect working tree"
        UNKNOWN_COUNT=$((UNKNOWN_COUNT + 1))
        return 0
    }
    tracked_changes=$(printf '%s\n' "$status" | grep -E -v '^$|^\?\? ' || true)
    untracked_files=$(printf '%s\n' "$status" | grep -E '^\?\? ' || true)
    [ -z "$tracked_changes" ] || add_reason "tracked modifications are present"
    [ -z "$untracked_files" ] || add_reason "untracked non-ignored files are present"

    ignored_lines=$(git -C "$workspace" status --porcelain=v1 --ignored=matching 2>/dev/null | sed -n 's/^!! //p') || {
        add_reason "ignored-file inspection failed"
        ignored_lines=""
    }
    while IFS= read -r ignored_path; do
        [ -n "$ignored_path" ] || continue
        safe_ignored_path "$ignored_path" || add_reason "ignored non-artifact path is present: $ignored_path"
    done <<<"$ignored_lines"

    if ! git -C "$workspace" rev-parse --verify "${head}^{commit}" >/dev/null 2>&1; then
        add_reason "HEAD is not a readable commit"
    elif ! commit_is_preserved_on_origin "$head"; then
        add_reason "detached or checked-out HEAD is not preserved on origin: $head"
    fi

    while IFS=' ' read -r ref sha; do
        [ -n "$ref" ] || continue
        if ! commit_is_preserved_on_origin "$sha"; then
            add_reason "local branch is not preserved on origin: ${ref#refs/heads/} ($sha)"
        fi
    done < <(git -C "$workspace" for-each-ref --format='%(refname) %(objectname)' refs/heads)

    while IFS=' ' read -r ref sha; do
        [ -n "$ref" ] || continue
        if ! commit_is_preserved_on_origin "$sha"; then
            add_reason "local tag is not preserved on origin: ${ref#refs/tags/} ($sha)"
        fi
    done < <(git -C "$workspace" for-each-ref --format='%(refname) %(objectname)' refs/tags)

    stash_count=$(git -C "$workspace" stash list 2>/dev/null | wc -l | trim_count)
    if [ "$stash_count" -gt 0 ] && [ "$relation" != registered ]; then
        add_reason "non-empty independent-clone stash is present"
    elif [ "$stash_count" -gt 0 ]; then
        # A registered worktree shares the authoritative common directory.  It
        # is safe to remove the worktree because this operation does not alter
        # refs/stash; the report makes that explicit rather than discarding it.
        :
    fi

    if [ -n "$REASONS" ]; then
        report_workspace UNSAFE "$workspace" "$kind" "$origin" "$branch" "$head" "$( [ -n "$status" ] && printf dirty || printf clean )" "$stash_count" "$artifact_kib" "retain: $REASONS"
        UNSAFE_COUNT=$((UNSAFE_COUNT + 1))
        return 0
    fi

    if [ "$stash_count" -gt 0 ]; then
        REASONS="safe: shared stash(es) remain in authoritative Git directory"
    else
        REASONS="safe: clean and every HEAD/branch/tag commit is preserved on origin"
    fi
    report_workspace SAFE "$workspace" "$kind" "$origin" "$branch" "$head" "clean" "$stash_count" "$artifact_kib" "$REASONS"
    SAFE_COUNT=$((SAFE_COUNT + 1))

    if [ "$MODE" = delete-safe ]; then
        if [ "$relation" = registered ]; then
            if git -C "$AUTH_ROOT" worktree remove "$workspace"; then
                say "DELETED|$workspace|method=git-worktree-remove"
                DELETED_COUNT=$((DELETED_COUNT + 1))
            else
                say "RETAINED|$workspace|reason=git worktree remove failed"
                UNSAFE_COUNT=$((UNSAFE_COUNT + 1))
            fi
        elif delete_independent_clone "$workspace"; then
            say "DELETED|$workspace|method=verified-independent-clone-removal"
            DELETED_COUNT=$((DELETED_COUNT + 1))
        else
            say "RETAINED|$workspace|reason=independent clone deletion safety guard failed"
            UNSAFE_COUNT=$((UNSAFE_COUNT + 1))
        fi
    fi
}

inspect_stale_worktrees() {
    local record_path="" record_head="" line
    local stale_reason
    inspect_one_stale() {
        [ -n "$record_path" ] || return 0
        if [ ! -e "$record_path" ] && is_below_tmp_root "$record_path"; then
            if [ -n "$record_head" ] && commit_is_preserved_on_origin "$record_head"; then
                printf 'STALE_WORKTREE|SAFE|path=%s|head=%s|action=%s\n' "$record_path" "$record_head" "metadata eligible for git worktree prune"
                SAFE_COUNT=$((SAFE_COUNT + 1))
            else
                stale_reason="missing worktree has an unreadable or origin-unpreserved HEAD"
                printf 'STALE_WORKTREE|UNSAFE|path=%s|head=%s|action=retain metadata: %s\n' "$record_path" "${record_head:-UNKNOWN}" "$stale_reason"
                UNSAFE_COUNT=$((UNSAFE_COUNT + 1))
                STALE_PRUNE_ALLOWED=0
            fi
        fi
    }

    while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in
            'worktree '*)
                inspect_one_stale
                record_path=${line#worktree }
                record_head=""
                ;;
            'HEAD '*)
                record_head=${line#HEAD }
                ;;
            '')
                inspect_one_stale
                record_path=""
                record_head=""
                ;;
        esac
    done < <(git -C "$AUTH_ROOT" worktree list --porcelain)
    inspect_one_stale
}

standalone_target_is_safe() {
    local target_dir=$1 entry fingerprint
    [ ! -L "$target_dir" ] || return 1
    [ -n "$AUTH_PACKAGE" ] || return 1
    [ -d "$target_dir/debug/.fingerprint" ] || [ -d "$target_dir/release/.fingerprint" ] || return 1
    fingerprint=$(find "$target_dir" -type d -path "*/.fingerprint/${AUTH_PACKAGE}-*" -print -quit 2>/dev/null)
    [ -n "$fingerprint" ] || return 1
    while IFS= read -r -d '' entry; do
        case "${entry##*/}" in
            debug|release|.rustc_info.json|CACHEDIR.TAG)
                ;;
            *)
                return 1
                ;;
        esac
    done < <(find "$target_dir" -mindepth 1 -maxdepth 1 -print0 2>/dev/null)
    return 0
}

inspect_standalone_targets() {
    local candidate
    while IFS= read -r -d '' candidate; do
        [ -d "$candidate" ] || continue
        [ -e "$candidate/.git" ] && continue
        if standalone_target_is_safe "$candidate"; then
            printf 'WORKSPACE|SAFE|path=%s|type=temporary non-Git Cargo target|size_kib=%s|origin=NONE|branch=NONE|head=NONE|state=standard Cargo target root|stashes=0|artifact_kib=%s|action=safe: package fingerprint and target-root shape prove generated build output\n' \
                "$candidate" "$(size_kib "$candidate")" "$(size_kib "$candidate")"
            SAFE_COUNT=$((SAFE_COUNT + 1))
            if [ "$MODE" = delete-safe ]; then
                if is_below_tmp_root "$candidate" && [ "$candidate" != "$TMP_ROOT" ] && [ ! -L "$candidate" ] && rm -rf -- "$candidate"; then
                    say "DELETED|$candidate|method=verified-standalone-cargo-target-removal"
                    DELETED_COUNT=$((DELETED_COUNT + 1))
                else
                    say "RETAINED|$candidate|reason=standalone Cargo target deletion safety guard failed"
                    UNSAFE_COUNT=$((UNSAFE_COUNT + 1))
                fi
            fi
        fi
    done < <(find "$TMP_ROOT" -mindepth 1 -maxdepth 1 -type d -print0 2>/dev/null)
}

# Review reports, patches, result JSON, and ad-hoc analysis files can be just
# as valuable as source.  They have no Git metadata to establish a durable
# relationship, so content evidence identifies them for an explicit UNKNOWN /
# retain result.  Content is never deleted by this utility.
temporary_entry_mentions_project() {
    local entry=$1
    if [ -f "$entry" ]; then
        rg -q -m 1 --max-filesize 2M -F "$AUTH_ROOT" "$entry" 2>/dev/null || \
            rg -q -m 1 --max-filesize 2M -F "$AUTH_PACKAGE" "$entry" 2>/dev/null
    elif [ -d "$entry" ]; then
        rg -q -m 1 --max-filesize 2M -g '!target/**' -g '!debug/**' -g '!release/**' -g '!.git/**' \
            -F "$AUTH_ROOT" "$entry" 2>/dev/null || \
            rg -q -m 1 --max-filesize 2M -g '!target/**' -g '!debug/**' -g '!release/**' -g '!.git/**' \
                -F "$AUTH_PACKAGE" "$entry" 2>/dev/null
    else
        return 1
    fi
}

inspect_non_git_project_artifacts() {
    local candidate
    [ -n "$AUTH_PACKAGE" ] || return 0
    while IFS= read -r -d '' candidate; do
        [ -e "$candidate/.git" ] && continue
        standalone_target_is_safe "$candidate" && continue
        if temporary_entry_mentions_project "$candidate"; then
            printf 'WORKSPACE|UNKNOWN|path=%s|type=temporary non-Git artifact|size_kib=%s|origin=NONE|branch=NONE|head=NONE|state=content references authoritative project|stashes=NONE|artifact_kib=0|action=retain: review/output artifact has no durable-history proof\n' \
                "$candidate" "$(size_kib "$candidate")"
            UNKNOWN_COUNT=$((UNKNOWN_COUNT + 1))
        fi
    done < <(find "$TMP_ROOT" -mindepth 1 -maxdepth 1 \( -type f -o -type d \) -print0 2>/dev/null)
}

inspect_stale_worktrees

# Find both conventional .git directories (independent clones) and .git files
# (linked worktrees).  The .git directory itself is pruned for performance.
while IFS= read -r -d '' dot_git; do
    inspect_git_workspace "${dot_git%/.git}"
done < <(find "$TMP_ROOT" \( -type d -name .git -prune -print0 -o -type f -name .git -print0 \) 2>/dev/null)

inspect_standalone_targets
inspect_non_git_project_artifacts

if [ "$MODE" = delete-safe ]; then
    if [ "$STALE_PRUNE_ALLOWED" -eq 1 ]; then
        if git -C "$AUTH_ROOT" worktree prune; then
            say "WORKTREE_PRUNE=completed"
        else
            say "WORKTREE_PRUNE=failed; stale metadata retained"
            UNSAFE_COUNT=$((UNSAFE_COUNT + 1))
        fi
    else
        say "WORKTREE_PRUNE=skipped because an origin-unpreserved stale worktree was found"
    fi
fi

say "SUMMARY|safe=$SAFE_COUNT|unsafe=$UNSAFE_COUNT|unknown=$UNKNOWN_COUNT|deleted=$DELETED_COUNT|mode=$MODE"
