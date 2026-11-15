# Git setup — fork model

Baseline commit: `9f37391eef056c0e6a45327b56670f200bb62460` (upstream `main`, release `v0.4.6`).
History was **not** rewritten: no squash, no rebase, no force-push, no `git init`.

## 1. Remote map

| Remote | URL | Role |
|---|---|---|
| `origin` | `https://github.com/bramburn/oh-my-android.git` | this fork (push target) |
| `upstream` | `https://github.com/ateymoori/oh-my-android.git` | the original (fetch only) |

Upstream identification: the only remote at clone time (`git remote -v`), matching
`project.yml`'s `SUFeedURL` (`ateymoori/oh-my-android`), `mcpb/manifest.json:12-17`,
`.claude-plugin/plugin.json`, and `README.md` release links. No ambiguity.

## 2. Executed (local only)

```sh
git remote rename origin upstream
git remote add origin https://github.com/bramburn/oh-my-android.git
git fetch upstream --tags
git branch upstream-main --track upstream/main
git tag -a upstream-baseline-9f37391 -m "Upstream baseline imported from ateymoori/oh-my-android@9f37391 (v0.4.6 + README tweak)"
git switch -c feat/windows-gui-port
git config core.autocrlf false # already false on this machine; set explicitly
```

Line endings: `core.autocrlf=false`, and `git ls-files --eol` shows **every tracked text file already
`w/lf`**, so adding `.gitattributes` (`* text=auto eol=lf`) renormalizes nothing. No `--renormalize`
was run and none is needed. If it ever becomes necessary, run it as its own commit and expect a large
diff.

## 3. Published on GitHub

The fork exists: **`bramburn/oh-my-android`** (fork of `ateymoori/oh-my-android`, public, default branch
`main`). What was executed, and what it means:

```sh
gh repo fork ateymoori/oh-my-android --remote=false   # create the fork, keep local remotes as-is
git switch main && git merge --ff-only feat/windows-gui-port
git push -u origin main                               # the Windows app lands on main
git tag -l 'upstream-baseline-*' -l 'v0.*' | xargs git push origin
```

`main` carries the fork's own work (the Windows app) and is **not** a mirror of upstream any more.
Upstream history stays available through `upstream` / `upstream-main`, and upstream *releases* remain
untouched (`v0.3.0`…`v0.4.6`, plus the `upstream-baseline-9f37391` tag). Do not delete those tags: the
fork point stays provable.

### Repository settings

| Setting | Value | Why |
|---|---|---|
| Default branch | `main` | `main` is the integration branch |
| Merge strategy | **squash only** | one commit per PR; no merge-commit noise from forks |
| Delete branch on merge | on | short-lived branches by construction |
| Auto-merge | on | the author can enable it once the required check passes |
| Issues | on, with templates | bug reports need structured data (adb version, `adb devices -l`) |
| Wiki / Projects | off | documentation lives in `docs/`, reviewed like code |
| License detection | MIT (from `LICENSE`) | unchanged upstream text |
| Dependency updates | Dependabot, weekly, grouped per ecosystem | pinned SHAs and crates go stale silently; every bump still needs `windows` + review |

### Branch protection on `main`

The maintainer can push directly; everybody else must raise a PR that passes CI. Applied with:

```sh
gh api -X PUT repos/bramburn/oh-my-android/branches/main/protection --input - <<'JSON'
{
  "required_status_checks": { "strict": true, "contexts": ["windows"] },
  "enforce_admins": false,
  "required_pull_request_reviews": {
    "required_approving_review_count": 1,
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false
  },
  "restrictions": { "users": ["bramburn"], "teams": [], "apps": [] },
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_conversation_resolution": true
}
JSON
```

- **`windows`** is the single required check — the job name in `.github/workflows/windows.yml`. That
  workflow therefore has **no `paths:` filter on `pull_request`**: a required check that never runs would
  block every merge.
- **`restrictions.users = [bramburn]`** is what makes "only I can work on main" true for everyone else;
  `enforce_admins: false` lets the maintainer bypass the PR requirement (admins are exempt unless forced).
- **Squash-only + linear history** keeps `main` a straight line, which makes `git log` and the release
  workflow's previous-tag lookup trivial.
- The macOS `build.yml` is **not** required: it is inherited from upstream and covers `Sources/`, which
  this fork does not ship.

Change the protection deliberately, not ad hoc: read it back with
`gh api repos/bramburn/oh-my-android/branches/main/protection`.

## 4. Branch model

| Branch | Purpose | Rules |
|---|---|---|
| `main` | integration branch; **protected** | PR required, `windows` check green, squash merge. Maintainer may push directly |
| `upstream-main` | read-only mirror of `upstream/main` | only ever fast-forwarded by `git merge --ff-only upstream/main` |
| `feat/<topic>` | one topic per branch | PR into `main`; delete after merge |
| `fix/<topic>` | bug fix | same |
| `docs/<topic>` | documentation only | same; must not touch `Sources/**` |
| `spike/<topic>` | throwaway experiments | never merged without an ADR |

Conventions are stated once in [`../CONTRIBUTING.md`](../CONTRIBUTING.md). Upstream's `badges` branch and
its download-counter workflow are not used in this fork (that workflow was removed); the Windows download
counter, if ever wanted, belongs to `release.yml`.

## 5. Sync-upstream playbook

```sh
# weekly, or before starting any port work
git fetch upstream --tags
git switch upstream-main
git merge --ff-only upstream/main          # if this fails, upstream rewrote history: stop and ask
git switch main
git merge upstream-main                    # normal merge: main carries local work, so not ff
git push origin main
```

Sync rules:

1. **Never** `reset --hard` on `main`. Upstream history is immutable; the fork's commits are not.
2. Tags from upstream are kept as-is (`v0.3.0`…`v0.4.6`). The fork's own releases use plain `vX.Y.Z`
   too, so check for a collision before tagging: `git tag -l "v$(node -p "require('./package.json').version")"`.
   Releases are created on Windows installers only (see `.github/workflows/release.yml`).
3. If `git merge --ff-only upstream/main` fails, upstream rewrote history. Do not `reset --hard`. Create an
   `upstream-main-recovered-<sha>` branch from the old tip, then ask for a decision.
4. Cherry-pick policy: cherry-picks are only for bringing a *single upstream fix* onto a port branch.
   Prefer merging upstream into `main`.
5. Conflict policy: if a conflict lands in `Sources/**` during the Windows re-implementation, upstream
   wins as the reference implementation — resolve by updating the fork's *new* code to match upstream
   behaviour, never by editing `Sources/**` to make the port easier.

## 6. Fork identity checklist (before any public release)

- [x] `LICENSE` unchanged (MIT, © 2026 Royan AB) — GitHub detects `MIT` for the fork.
- [ ] `NOTICE.md` updated with every source change since the fork point.
- [x] New name/icon/bundle id — do not reuse "Oh My Android", `se.royan.ohmyandroid`, the icon, or the tap.
- [ ] New update feed + new signing key (never `SUPublicEDKey` from `project.yml:58`).
- [ ] Upstream's MCP packaging files (`mcpb/manifest.json`, `.claude-plugin/plugin.json`, `server.json`,
  `glama.json`) are inert here — the MCP server is **not** ported; leave them untouched rather than
  repointing them at a fork that does not implement them.

## 7. Verification commands

```sh
gh repo view bramburn/oh-my-android --json isFork,defaultBranchRef,licenseInfo
gh api repos/bramburn/oh-my-android/branches/main/protection --jq '.required_status_checks.contexts'
gh run list --limit 5                       # CI after the first push
git log --oneline --decorate -3
git diff --stat upstream-baseline-9f37391 -- Sources   # must be empty, always
git diff --quiet upstream-baseline-9f37391 -- LICENSE && echo "LICENSE byte-identical"
git ls-files --eol | grep -v w/lf           # text files should all be LF
```