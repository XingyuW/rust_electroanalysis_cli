#!/usr/bin/env bash
# Isolated regression coverage for scripts/cleanup_codex_tmp.sh.

set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel)
TOOL="$REPO_ROOT/scripts/cleanup_codex_tmp.sh"
FIXTURE_ROOT=$(mktemp -d /private/tmp/cleanup-codex-tmp-test.XXXXXX)
REMOTE="$FIXTURE_ROOT/origin.git"
AUTH="$FIXTURE_ROOT/authoritative"
SCAN="$FIXTURE_ROOT/scan root"

cleanup() {
    if [ "${KEEP_FIXTURES:-0}" = 1 ]; then
        printf 'retained fixture root for debugging: %s\n' "$FIXTURE_ROOT" >&2
    else
        rm -rf -- "$FIXTURE_ROOT"
    fi
}
trap cleanup EXIT HUP INT TERM

fail() {
    printf 'TEST FAILURE: %s\n' "$*" >&2
    exit 1
}

assert_exists() {
    [ -e "$1" ] || fail "expected retained path does not exist: $1"
}

assert_absent() {
    [ ! -e "$1" ] || fail "expected deleted path still exists: $1"
}

assert_reported_as() {
    local output=$1 path=$2 classification=$3
    grep -F "path=$path" "$output" | grep -F "|$classification|" >/dev/null || fail "$path was not reported as $classification"
}

make_clean_clone() {
    local destination=$1
    git clone -q "$REMOTE" "$destination"
    git -C "$destination" config user.name "Cleanup Fixture"
    git -C "$destination" config user.email "cleanup-fixture@example.invalid"
}

make_target() {
    local destination=$1
    mkdir -p "$destination/release/.fingerprint/rust_electroanalysis_cli-fixture" "$destination/release/deps"
    touch "$destination/release/.fingerprint/rust_electroanalysis_cli-fixture/invoked.timestamp"
    # A sparse file makes the regression fixture substantial without slowing it
    # down; it models a large generated release artifact.
    truncate -s 64M "$destination/release/deps/rust_electroanalysis_cli-fixture"
}

run_tool() {
    local output=$1
    shift
    (
        cd "$AUTH"
        CODEX_TMP_ROOT="$SCAN" bash "$TOOL" "$@"
    ) >"$output"
}

mkdir -p "$SCAN"
git init --bare -q "$REMOTE"
git init -q "$AUTH"
git -C "$AUTH" config user.name "Cleanup Fixture"
git -C "$AUTH" config user.email "cleanup-fixture@example.invalid"
mkdir -p "$AUTH/src"
printf '[package]\nname = "rust_electroanalysis_cli"\nversion = "0.1.0"\nedition = "2024"\n' >"$AUTH/Cargo.toml"
touch "$AUTH/tracked.txt"
git -C "$AUTH" add Cargo.toml tracked.txt
git -C "$AUTH" commit -qm "fixture baseline"
git -C "$AUTH" branch -M main
git -C "$AUTH" remote add origin "$REMOTE"
git -C "$AUTH" push -qu origin main
git -C "$REMOTE" symbolic-ref HEAD refs/heads/main

# Positive cases: clean clone, registered worktree, package-identifying Cargo
# target, a branch whose remote was deleted, tag-only preservation, and a
# second remote-tracking branch.
CLEAN="$SCAN/clean clone with spaces"
REGISTERED="$SCAN/registered worktree"
TARGET="$SCAN/large target"
SCAFFOLD_TARGET="$SCAN/empty source scaffold"
SNAPSHOT="$SCAN/github identical snapshot.toml"
DELETED_REMOTE="$SCAN/deleted remote branch"
TAG_ONLY="$SCAN/tag only"
MULTI_REMOTE="$SCAN/multiple remote branches"
make_clean_clone "$CLEAN"
git -C "$AUTH" worktree add --detach -q "$REGISTERED" main
make_target "$TARGET"
mkdir -p "$SCAFFOLD_TARGET/src" "$SCAFFOLD_TARGET/docs"
make_target "$SCAFFOLD_TARGET/target"
cp "$AUTH/Cargo.toml" "$SNAPSHOT"

git -C "$AUTH" branch deleted-remote main
git -C "$AUTH" push -q origin deleted-remote
make_clean_clone "$DELETED_REMOTE"
git -C "$AUTH" push -q origin --delete deleted-remote

TAG_SEED="$FIXTURE_ROOT/tag-seed"
make_clean_clone "$TAG_SEED"
git -C "$TAG_SEED" checkout -qb tag-only
git -C "$TAG_SEED" commit --allow-empty -qm "tag-only preserved commit"
git -C "$TAG_SEED" tag tag-only-preserved
git -C "$TAG_SEED" push -q origin refs/tags/tag-only-preserved
make_clean_clone "$TAG_ONLY"
git -C "$TAG_ONLY" fetch -q --tags origin
git -C "$TAG_ONLY" checkout -q --detach tag-only-preserved

git -C "$AUTH" branch release/parallel main
git -C "$AUTH" push -q origin release/parallel
make_clean_clone "$MULTI_REMOTE"
git -C "$MULTI_REMOTE" checkout -q --detach origin/release/parallel

# Negative cases: unstaged/staged tracked changes, valuable untracked source,
# local-only HEAD/branch history, detached unique history, stash, different
# origin, and an ambiguous project-shaped repository.
UNSTAGED="$SCAN/unstaged tracked change"
STAGED="$SCAN/staged tracked change"
UNTRACKED="$SCAN/valuable untracked source"
LOCAL_ONLY="$SCAN/local only commit"
DETACHED="$SCAN/detached unique commit"
UNIQUE_BRANCH="$SCAN/unique local branch"
STASHED="$SCAN/non-empty stash"
DIFFERENT_ORIGIN="$SCAN/different origin"
AMBIGUOUS="$SCAN/ambiguous project repository"
UNRELATED="$SCAN/unrelated repository"
EMPTY="$SCAN/empty directory"
REVIEW_OUTPUT="$SCAN/review output.md"

make_clean_clone "$UNSTAGED"
rm "$UNSTAGED/tracked.txt"
make_clean_clone "$STAGED"
rm "$STAGED/tracked.txt"
git -C "$STAGED" add -u
make_clean_clone "$UNTRACKED"
touch "$UNTRACKED/valuable.rs"
make_clean_clone "$LOCAL_ONLY"
git -C "$LOCAL_ONLY" commit --allow-empty -qm "local-only commit"
make_clean_clone "$DETACHED"
git -C "$DETACHED" checkout -q --detach
git -C "$DETACHED" commit --allow-empty -qm "detached unique commit"
make_clean_clone "$UNIQUE_BRANCH"
git -C "$UNIQUE_BRANCH" checkout -qb fixture-unique-branch
git -C "$UNIQUE_BRANCH" commit --allow-empty -qm "unique branch commit"
make_clean_clone "$STASHED"
rm "$STASHED/tracked.txt"
git -C "$STASHED" stash push -qm "fixture stash"

OTHER_REMOTE="$FIXTURE_ROOT/different-origin.git"
git init --bare -q "$OTHER_REMOTE"
make_clean_clone "$DIFFERENT_ORIGIN"
git -C "$DIFFERENT_ORIGIN" remote set-url origin "$OTHER_REMOTE"
make_clean_clone "$AMBIGUOUS"
git -C "$AMBIGUOUS" remote remove origin
git init -q "$UNRELATED"
touch "$UNRELATED/unrelated.txt"
mkdir -p "$EMPTY"
printf 'review evidence for rust_electroanalysis_cli\n' >"$REVIEW_OUTPUT"

# A missing registered worktree must be pruned only after its HEAD is proven
# reachable from origin.
STALE="$SCAN/stale registered worktree"
git -C "$AUTH" worktree add --detach -q "$STALE" main
rm -rf -- "$STALE"

DRY_OUTPUT="$FIXTURE_ROOT/dry-run.txt"
run_tool "$DRY_OUTPUT"

for path in "$CLEAN" "$REGISTERED" "$TARGET" "$SCAFFOLD_TARGET" "$SNAPSHOT" "$DELETED_REMOTE" "$TAG_ONLY" "$MULTI_REMOTE"; do
    assert_reported_as "$DRY_OUTPUT" "$path" SAFE
done
for path in "$UNSTAGED" "$STAGED" "$UNTRACKED" "$LOCAL_ONLY" "$DETACHED" "$UNIQUE_BRANCH" "$STASHED"; do
    assert_reported_as "$DRY_OUTPUT" "$path" UNSAFE
done
for path in "$DIFFERENT_ORIGIN" "$AMBIGUOUS"; do
    assert_reported_as "$DRY_OUTPUT" "$path" UNKNOWN
done
assert_reported_as "$DRY_OUTPUT" "$REVIEW_OUTPUT" UNKNOWN
grep -F "STALE_WORKTREE|SAFE|path=$STALE" "$DRY_OUTPUT" >/dev/null || fail "stale worktree was not proven safe"
assert_exists "$CLEAN"
assert_exists "$TARGET"
assert_exists "$UNRELATED"
assert_exists "$EMPTY"

DELETE_OUTPUT="$FIXTURE_ROOT/delete-safe.txt"
run_tool "$DELETE_OUTPUT" --delete-safe

for path in "$CLEAN" "$REGISTERED" "$TARGET" "$SCAFFOLD_TARGET" "$SNAPSHOT" "$DELETED_REMOTE" "$TAG_ONLY" "$MULTI_REMOTE"; do
    assert_absent "$path"
done
for path in "$UNSTAGED" "$STAGED" "$UNTRACKED" "$LOCAL_ONLY" "$DETACHED" "$UNIQUE_BRANCH" "$STASHED" "$DIFFERENT_ORIGIN" "$AMBIGUOUS" "$UNRELATED" "$EMPTY" "$REVIEW_OUTPUT"; do
    assert_exists "$path"
done
if git -C "$AUTH" worktree list --porcelain | grep -F "worktree $STALE" >/dev/null; then
    fail "safe stale worktree metadata was not pruned"
fi

# Fetch failure is a hard gate: the tool must not delete a clean workspace.
FETCH_FAILURE="$SCAN/fetch failure retained"
make_clean_clone "$FETCH_FAILURE"
git -C "$AUTH" remote set-url origin "$FIXTURE_ROOT/missing-origin.git"
if (
    cd "$AUTH"
    CODEX_TMP_ROOT="$SCAN" bash "$TOOL" --delete-safe
) >"$FIXTURE_ROOT/fetch-failure.txt" 2>&1; then
    fail "fetch-failure fixture unexpectedly succeeded"
fi
grep -F 'NO-GO: origin fetch failed' "$FIXTURE_ROOT/fetch-failure.txt" >/dev/null || fail "fetch failure did not report NO-GO"
assert_exists "$FETCH_FAILURE"

printf 'cleanup_codex_tmp.sh fixture tests: PASS\n'
