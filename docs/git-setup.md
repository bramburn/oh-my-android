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

## 2. Executed (local only, no network writes)

```sh
git remote rename origin upstream
git remote add origin https://github.com/bramburn/oh-my-android.git
git fetch upstream --tags
git branch upstream-main --track upstream/main
git tag -a upstream-baseline-9f37391 -m "Upstream baseline imported from ateymoori/oh-my-android@9f37391 (v0.4.6 + README tweak)"
git switch -c feat/windows-gui-port
git config core.autocrlf false     # already false on this machine; set explicitly
```

State after these commands:

```
* feat/windows-gui-port  9f37391   (working branch for the fork's work)
  main                   9f37391   [upstream/main]   (untouched upstream mirror; retarget to origin/main after the fork push)
  upstream-main          9f37391   [upstream/main]   (pristine mirror of upstream's default branch)
tag: upstream-baseline-9f37391
```

Line endings: `core.autocrlf=false`, and `git ls-files --eol` shows **every tracked text file already
`w/lf`**, so adding `.gitattributes` (`* text=auto eol=lf`) renormalizes nothing. No `--renormalize`
was run and none is needed. If it ever becomes necessary, run it as its own commit and expect a large
diff — do not mix it with a feature commit.

`git config core.excludesFile` / `user.*` were not touched.

## 3. Pending — needs approval (Gate 1)

```sh
# 1. Create the fork on GitHub (nothing is pushed by this command).
gh repo fork ateymoori/oh-my-android --clone=false --remote=false

# 2. Publish the untouched upstream mirror + the baseline tag.
git push -u origin main
git push origin upstream-main
git push origin upstream-baseline-9f37391

# 3. Publish the documentation branch.
git push -u origin feat/windows-gui-port

# 4. After the fork exists, make local main track the fork, not upstream.
git branch --set-upstream-to=origin/main main
```

Alternative if `gh` is unavailable: create the fork in the GitHub UI ("Fork" on
`ateymoori/oh-my-android`), then run steps 2-4 unchanged.

Credentials on this machine: `gh` is authenticated as `bramburn` (verified with `gh auth status`);
`gh repo view bramburn/oh-my-android` currently returns "Could not resolve to a Repository", i.e. the
fork **does not exist yet**.

## 4. Branch model

| Branch | Purpose | Rules |
|---|---|---|
| `main` | this fork's integration branch; mirrors upstream between merges | fast-forward from `upstream-main` or merges of feature branches; never force-push |
| `upstream-main` | read-only mirror of `upstream/main` | only ever fast-forwarded by `git merge --ff-only upstream/main` |
| `feat/<topic>` | one topic per branch, e.g. `feat/windows-gui-port` | PR into `main`; delete after merge |
| `docs/<topic>` | documentation-only work | same rules; must not touch `Sources/**` |
| `spike/<topic>` | throwaway experiments (mirroring, framework) | never merged without an ADR |

Prefixes match `docs/CONTRIBUTING-fork.md`. Upstream's own branch names (`badges`, `launch/readme-hn`)
are not used in this fork; `badges` is CI-generated (`downloads.yml`).

## 5. Sync-upstream playbook

```sh
# weekly, or before starting any port work
git fetch upstream --tags
git switch upstream-main
git merge --ff-only upstream/main          # if this fails, upstream rewrote history: stop and ask
git switch main
git merge --ff-only upstream-main          # only if main has no local commits
git switch feat/windows-gui-port
git merge main                             # or: git rebase main (allowed on private branches only)
```

Rules:

1. **Never** rebase or force-push `main` or `upstream-main`.
2. **Never** squash upstream commits when integrating — merge (or cherry-pick with the original message
   preserved) so attribution survives.
3. If `git merge --ff-only upstream/main` fails, upstream rewrote history. Do not `reset --hard`. Create a
   `upstream-main-recovered-<sha>` branch from the old tip, then ask for a decision.
4. Cherry-pick policy: cherry-picks are only for bringing a *single upstream fix* onto a port branch.
   Prefer merging upstream into `main` and then merging `main` down.
5. Conflict policy: if a conflict lands in `Sources/**` during the Windows re-implementation, upstream
   wins as the reference implementation — resolve by updating the fork's *new* code to match upstream
   behaviour, never by editing `Sources/**` to make the port easier.
6. Tags from upstream are kept as-is (`v0.3.0`…`v0.4.6`). The fork's own releases use a distinct prefix
   (proposal: `win-v0.1.0`) so upstream and fork releases never collide.

## 6. Fork identity checklist (before any public release)

- [ ] `LICENSE` unchanged (MIT, © 2026 Royan AB).
- [ ] `NOTICE.md` updated with every source change since the fork point.
- [ ] New name/icon/bundle id — do not reuse "Oh My Android", `se.royan.ohmyandroid`, the icon, or the tap.
- [ ] New update feed + new signing key (never `SUPublicEDKey` from `project.yml:58`).
- [ ] `mcpb/manifest.json`, `.claude-plugin/plugin.json`, `server.json`, `glama.json` repointed at the fork
      (and `compatibility.platforms` extended to `win32` if the server is ported).

## 7. Verification commands

```sh
git log --oneline --decorate -3          # fork point + doc commits
git diff --stat upstream-baseline-9f37391 -- Sources   # must be empty in the docs phase
git diff --quiet upstream-baseline-9f37391 -- LICENSE && echo "LICENSE byte-identical"
git ls-files --eol | grep -v w/lf        # text files should all be LF
```