# Safe Codex temporary-workspace cleanup

Run the audit from the authoritative checkout:

```bash
./scripts/cleanup_codex_tmp.sh
```

The default is dry-run only. It fetches `origin`, verifies remote heads and
tags, then reports every related temporary workspace as `SAFE`, `UNSAFE`, or
`UNKNOWN`.

After reviewing that report, remove only the proven-safe entries:

```bash
./scripts/cleanup_codex_tmp.sh --delete-safe
```

The utility fails closed. In particular, it retains any workspace with tracked
changes, non-ignored untracked files, local-only branch/tag/HEAD history, an
independent-clone stash, an ambiguous repository relationship, or a
project-referencing non-Git artifact. Registered-worktree stashes are reported
but retained in the authoritative Git directory; removing such a worktree does
not alter `refs/stash`.

It only deletes a standalone non-Git directory when its Cargo fingerprint
identifies this package and its top-level layout is a standard `target/` root.
All other non-Git project artifacts, including plans, review reports, patches,
and result outputs, remain `UNKNOWN` until separately reconciled.

The fixture suite can be run with:

```bash
bash tests/cleanup_codex_tmp.sh
```

It covers clean clones/worktrees and generated target artifacts, plus tracked
and untracked changes, stashes, local-only and detached commits, tag-only
history, a deleted remote branch, unrelated/different-origin repositories,
ambiguous relationships, a stale worktree, spaces in paths, and fetch failure.
