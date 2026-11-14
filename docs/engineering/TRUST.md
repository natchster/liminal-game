# The maintainer trust: whose words the agents act on, and what only the owner changes

This repository is public, and its agents implement GitHub issues. Anything
an agent reads can steer it, so the rule is simple: **an agent acts on the
words of the owner and maintainers, and on others' words only once a
maintainer has approved them as they stand.** The agents' own skills,
hooks, settings, workflows and this trust tooling (the *protected paths*)
change only by the owner and maintainers. When a maintainer's work came
from someone else's words, those changes also need an explicit approval of
the exact commit. No existing skill or integration could be set up this way
([`../research/agent-trust.md`](../research/agent-trust.md)), so this is the
project's own.

| Part | Where | Does |
|---|---|---|
| the policy | `.github/trust/policy.json` | trusted roles and users, the protected paths |
| the trust tool | `.github/trust/trust.py` (tests: `.github/trust/test_trust.py`, a step of `cargo xtask check`) | the trusted brief of an issue, the gate's verdict on a pull request, the CI agent's go-ahead |
| the skill | `.claude/skills/implement-issue/SKILL.md` | how an agent plans and implements an issue from its brief |
| the trust gate | `.github/workflows/trust-gate.yml` | the `trust-gate` status on every pull request |
| the agent on GitHub | `.github/workflows/claude.yml` | `@claude`, behind the same rules |
| the pre-edit hook | `xtask/src/hook.rs` | refuses protected-path edits in a session working from others' words |
| the repository's settings | `.github/trust/ruleset.json`, `.github/trust/actions-policy.json`, `.github/CODEOWNERS` | make `trust-gate` required, let it run, give the protected paths an owner |

## Who is trusted

A **maintainer** is anyone whose repository role is in `trusted_roles`
(`admin`, `maintain`), from the collaborator permission API, or who is named
in `trusted_users`. A personal repository has no maintain role, so today
that is the owner alone. Write collaborators, anyone else, deleted accounts
and **bots** are not trusted. A bot's words speak for whoever ran it, so
they neither count as a maintainer's nor make a thread "someone else's".

## What maintainers write

Each command goes on a line of its own in a conversation comment. Commands
inside code blocks or quotes, in reviews, by anyone who is not a
maintainer, or in a maintainer's comment that someone else edited, do not
count.

| Command | Means |
|---|---|
| `/trust` | the title, the body and every comment so far are approved |
| `/trust body` | only the title and the body |
| `/trust <comment link> …` | only those comments (`#issuecomment-…`, `#discussion_r…`, `#pullrequestreview-…`) |
| `/revoke` | every approval (`/trust`, `/trust-protected`) made before it is withdrawn |
| `/trust-protected <sha>` | on a pull request: its protected-path changes are allowed at exactly this head commit (7 or more hex digits) |

**An approval covers words as they were when it was given.** Words count as
approved only when their last change happened before the approval: the body
or comment's `lastEditedAt`, or for an issue, a title change by someone
other than a maintainer. An approval counts from its own last edit. Edit
approved words afterwards and they are untrusted again until a new
`/trust`. A `/trust-protected` names a commit, so any push needs a new one.
Plain words ("LGTM", "approved") approve nothing.

## How an agent reads an issue: its trusted brief

`python3 .github/trust/trust.py issue N` (the skill's first step) is the only
way an agent reads an issue or a pull request's conversation. It prints:

- the **standing**: `maintainer` (a maintainer opened it), `approved`
  (someone else did, and a maintainer approved it), or `untrusted`
  (BLOCKED: the agent stops, and the title is withheld too). Exit 0 when
  implementable, 2 when blocked;
- the **provenance**: `maintainer` when every non-bot word in the thread is
  a maintainer's, otherwise `third-party`, with the reasons. Words a
  maintainer approved still count as someone else's for provenance
  (`provenance_includes_comments`). Approval lets an agent act on them; it
  does not let them steer protected paths;
- the trusted words, each with its author, role and times. What a person
  reading the page cannot see is removed first: HTML comments, invisible
  and direction characters, the Unicode tag block, hidden link
  definitions, hidden HTML attributes, image alt texts;
- what was **withheld**: who, when and why, never what.

`--mark` adds the issue to `target/trust/task.json` when its provenance is
third-party and takes it out when it is the maintainers'. It never touches
another issue's entry, so briefing a maintainer's issue cannot lift the
marker an outsider's issue set. The file goes away with its last entry.
`--json` gives the same as data.

## The trust gate

Every pull request event (and every `/trust-protected` or `/revoke` comment
on one) runs `trust.py pr N --publish`, which sets the `trust-gate` commit
status on the head commit and keeps one comment explaining it:

| The pull request | `trust-gate` |
|---|---|
| changes no protected path | success |
| changes one, and its author is not a maintainer (write collaborators included) | **failure, rejected**: nothing a maintainer says changes it; only the owner's ruleset bypass can merge it |
| changes one; a maintainer opened it, and nothing it implements comes from outside | success |
| changes one; a maintainer (or a bot) opened it, and words or commits come from outside | failure until a maintainer comments `/trust-protected <head sha>` |

"From outside" means any of these:

- an issue or pull request it names was opened by a non-maintainer, or has
  non-maintainer comments. The names come from closing references, `#N`
  and links in the title, body and commit messages, and `issue-N` in the
  branch name (the CI agent names its branches `claude/issue-N-…`);
- its own conversation has non-maintainer words;
- a commit is by a non-maintainer (GitHub's `web-flow` and bots aside), or
  its author's email matches no GitHub account;
- it was opened by a bot;
- something cannot be read. Too many files, 250+ commits, more than 50
  references, or an unreadable reference all fail closed.

Paths are matched the way a case-insensitive disk would write them
(`.Claude/x` lands in `.claude/` on a Mac), with invisible characters
ignored. A rename counts by both its old and new names.

The workflow is safe by construction. `pull_request_target` and
`issue_comment` always run the workflow file and the trust tool as they are
on the default branch. The pull request's code is never checked out, and its
title and body never reach a shell. A pull request that edits the gate
changes nothing until it is merged. The gate posts `pending` first, so a
fork's own workflow cannot satisfy the required check with a check run of
the same name in the meantime.

## The agent on GitHub

`claude.yml` runs a `trust` job before the agent (`trust.py ci N --actor …`):

- only a maintainer's mention starts it. Anyone else's is ignored, without
  a reply;
- the action always reads the opening post, so that post must be trusted.
  Otherwise the maintainer is told why, and nothing runs;
- authors of withheld comments and reviews are passed to the action's
  `exclude_comments_by_actor`, so the action does not read them;
- the agent is denied `gh`, WebFetch and WebSearch (with them it could read
  withheld words). When the provenance is third-party, its edit tools are
  also denied the protected paths.

The action's own protections still apply on top: content created or edited
after the trigger is dropped, the body is the webhook's snapshot, and
`.claude/` and `CLAUDE.md` are restored from the base branch on pull requests.

## In a session: the pre-edit hook

While `target/trust/task.json` exists, the pre-edit hook refuses edits to
the protected paths it lists, saying which issue and why. It covers the
editing tools, not the shell, so it stops a mistake where it starts. The gate
is the line that holds. Only the owner clears the marker
(`python3 .github/trust/trust.py clear`), when the session is no longer
working on that issue. A brief of the same issue, once its words are all
maintainers', removes that issue's entry.

## The protected paths

| Path | Why |
|---|---|
| `.claude/**` | skills, reviewers, rules, hooks and settings: what agents load as instructions and run as commands |
| `.github/**` | the workflows (with their secrets), the trust tool, its policy, `CODEOWNERS` |
| `CLAUDE.md`, `**/CLAUDE.md`, `AGENTS.md`, `.mcp.json` | instructions and tools agents load by themselves |
| `.cargo/**` | `config.toml` defines the `xtask` alias every hook runs |
| `xtask/src/hook.rs` | what the hooks do |

The rest of `xtask/` is ordinary code, reviewed like the game's.

## The repository's settings

These are done once, by the owner (rulesets and Actions policies are
admin-only, and no workflow token can change them). Both need this
workflow on the default branch first.

1. **Let the gate run.** From 2026-12-02 public repositories block
   `pull_request_target` unless an Actions event policy allows it, and the
   2026-11-16 brownout already fails such runs for two hours. The policy
   allows it, and `issue_comment`, for `trust-gate.yml` only (Settings →
   Actions → Policies, or
   `gh api -X POST repos/OWNER/REPO/actions/policies --input .github/trust/actions-policy.json`).
2. **Make it required.** The ruleset on the default branch requires a pull
   request, code-owner review and the `trust-gate` status from GitHub
   Actions (`integration_id` 15368), and forbids deleting the branch or
   force-pushing it. The admin role bypasses it, so the owner can still
   push directly and can deliberately merge past a red gate
   (`gh api -X POST repos/OWNER/REPO/rulesets --input .github/trust/ruleset.json`).
   With `bypass_mode` `pull_request` instead of `always`, the owner works
   through pull requests too.
3. **Forks' workflows wait for approval.** Settings → Actions → General →
   "Require approval for all external contributors". A fork's changed
   workflow then cannot run, or impersonate a check, unseen.

After the first gate run, check that its status shows "GitHub Actions" as
its source in the ruleset's status picker.

## Why Python

`CLAUDE.md`'s automation is Rust, in `xtask`. The trust tool is the
exception because it runs inside a required, fail-closed check on GitHub's
Linux runners, from the default branch. As `xtask` it would need a Rust
toolchain and a C++ build (`nv-flip`) before every verdict, and minutes
after every `/trust-protected`. Python's standard library and `gh` are
already on the runner, and the tool is one file that is easy to audit. Its
tests run in `cargo xtask check` like everything else. The pre-edit hook
matches globs in Rust the same way (`glob_match`, tested against the same
cases).

## What it does not cover

- **People with write access.** They can push branches whose workflows run
  with the repository's secrets, and can post statuses. Today nobody but
  the owner has it. Grant it sparingly, and only `maintain` makes someone
  trusted here.
- **The owner's own bypass and direct pushes.** These are the owner's
  decisions. The hook and the skill keep an agent from making protected
  changes from others' words in a session, but a person can still commit
  them.
- **Words outside GitHub threads.** A linked page, a file in a pull request,
  or a dependency are data the agent may read. The skill tells it to take
  them as data, but no tool filters them.
- **A race at approval time.** A maintainer who reads a body, sees it
  edited in the seconds before they comment, and approves anyway, approves
  the edit. Read the thread again after an "edited" marker.
- **Other repositories' issues** (`owner/repo#N`). The gate reads only this
  repository's references.
