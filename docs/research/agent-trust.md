# Agents and other people's words: what exists (2026-11-14)

The brief: this repository is public, and its agents implement GitHub
issues. Can an existing skill or integration be set so that only the
owner's and maintainers' words (role `admin` or `maintain`) are acted on,
others' words only once a maintainer approves them (and no longer once they
are edited), and the agents' own skills and checks change only by the
owner, even when a maintainer's pull request implements someone else's
issue? Gathered by research agents from the tools' source code and GitHub's
documentation; what could not be verified says so. The design that answers
it is [`../engineering/TRUST.md`](../engineering/TRUST.md).

## The answer

Nothing found does all of it. Every integration trusts **write** access at
best, none ties an approval given in a maintainer's comment to the version
of the words approved, and none refuses a change to the agent's own
configuration because of who wrote the issue behind it. One idea is worth
borrowing from each of three: gh-aw's endorsement that goes stale when the
endorsed item is edited, claude-code-action's freezing of the thread at
trigger time, and gh-aw's protected files.

| Tool | Tells maintainers from others | Maintainer approval | Edit withdraws approval | Protects its own configuration |
|---|---|---|---|---|
| Claude Code skills and plugins (official marketplace, anthropics/skills, superpowers, ccpm, feiskyer/github-fix-issue) | no | no | no | no |
| anthropics/claude-code-action v1.0.276 | the trigger only (write or above) | no | content after the trigger dropped | restores `.claude/`, `CLAUDE.md`, `.mcp.json` from the base branch for what it reads, not what it writes |
| GitHub MCP server, lockdown mode | push access (write) | no | no | no |
| GitHub Agentic Workflows (gh-aw) v1.2.2 | `author_association` (write level) | labels; reactions (opt-in) | reactions yes, labels no | protected files: request a review by default, or block, whoever wrote the issue |
| Copilot cloud agent | write access; others' comments never shown | no | not documented | not documented |
| OpenCode, Gemini CLI, Codex actions | the trigger only (write or above) | no | no | no |
| OpenHands' issue resolver | treats all issue text as untrusted | no | no | in its prompt only |

## claude-code-action

Read from source on `main` ([`action.yml`](https://github.com/anthropics/claude-code-action/blob/main/action.yml),
[its security guide](https://github.com/anthropics/claude-code-action/blob/main/docs/security.md),
`src/github/data/fetcher.ts`, `src/github/utils/sanitizer.ts`,
`src/modes/detector.ts`).

- **Who triggers:** the actor (commenter, labeller, assigner) must have
  `admin` or `write` from the legacy permission API, which reports
  `maintain` as `write`: plain write users pass, and it cannot be narrowed
  to admin and maintain. The issue's author is never checked.
  `allowed_non_write_users` widens it; `allowed_bots` lets bots trigger.
- **What it reads:** the title, the body, the first 100 comments, reviews
  and review comments, changed files, and their images. **The body is
  always included, whoever wrote it.** `include_comments_by_actor` and
  `exclude_comments_by_actor` (v1.0.37) filter comments and reviews by
  exact login (`*[bot]` the only wildcard), not the body. Minimized
  comments are dropped.
- **Edits:** comments created or edited at or after the trigger are
  dropped; the body comes from the webhook's snapshot (v1.0.45). Edits made
  before the trigger are accepted; there is no notion of approval.
- **Sanitizing:** HTML comments, zero-width, control and bidi characters,
  image alt texts, link titles, hidden attributes (`alt`, `title`,
  `aria-label`, `data-*`, `placeholder`) and GitHub tokens are removed.
- **Agent mode:** with `prompt` set on an issue or pull request event it
  fetches nothing from GitHub and uses the prompt as given. `GH_TOKEN` is in
  the agent's environment, so `gh` can still read anything unless denied
  (its security guide).
- **Its own files:** on pull requests, `.claude/`, `CLAUDE.md` and
  `.mcp.json` are restored from the base branch before the agent reads
  them; the Claude GitHub App cannot write workflows (its FAQ).

## GitHub MCP server: lockdown mode

[README](https://github.com/github/github-mcp-server), `pkg/lockdown/lockdown.go`;
v0.21.0 (2025-11-17), `--lockdown-mode` or `GITHUB_LOCKDOWN_MODE=1`.
Content is shown when its author has push access (admin or write; maintain
counts as write), is `github-actions[bot]` or Copilot, is the token's user,
or the repository is private. Reading an untrusted author's issue fails
whole; their comments and reviews are filtered out; `list_issues` and
`search_issues` are not filtered. No approval, no edit times, and in its
own words not an authorization boundary. claude-code-action pins the
server at v0.17.1 (before lockdown) and does not turn it on.

## GitHub Agentic Workflows

[Integrity](https://github.github.com/gh-aw/reference/integrity/),
[safe outputs for pull requests](https://github.github.com/gh-aw/reference/safe-outputs-pull-requests/),
the guard in [gh-aw-mcpg](https://github.com/github/gh-aw-mcpg)
(`guards/github-guard/rust-guard/src/labels/helpers.rs`).

- `on.roles` (default `admin, maintainer, write`) gates the trigger only.
- `tools.github.min-integrity` (`merged`, `approved`, `unapproved`, `none`;
  public repositories default to `approved`) filters each item before the
  engine sees it, by `author_association`: OWNER, MEMBER and COLLABORATOR
  are `approved`, falling back to `admin`, `maintain` or `write`. Write
  users therefore count as maintainers.
- `approval-labels` promote a labelled item without any edit check.
  Endorsement reactions (`features.integrity-reactions`, v0.68.2) are
  skipped when the item was edited after the reaction ([gh-aw-mcpg
  #4228](https://github.com/github/gh-aw-mcpg/pull/4228), 2026-04-21): the
  one mechanism found that withdraws an approval on edit.
- `safe-outputs.create-pull-request.protected-files`: `request-review`
  (default), `blocked`, `fallback-to-issue`, `allowed`; protects manifests,
  `CODEOWNERS`, `AGENTS.md`, `CLAUDE.md` and `.claude/` (under the Claude
  engine), `.github/` and every top-level dot folder. It applies to the
  agent's own pull requests whoever wrote the issue, and not at all to a
  person's pull request.
- Unverified: whether the triggering event's own text, put into the prompt,
  passes through the integrity filter.

## Others

- **Copilot cloud agent** ([risks and mitigations](https://docs.github.com/en/copilot/concepts/agents/cloud-agent/risks-and-mitigations)):
  only write users trigger it; "comments from users without write access
  are never presented to the agent"; hidden characters and HTML comments
  filtered. The issue body's treatment is not stated.
- **OpenCode** (`github.handler.ts` at fc7e4cf): the actor must be `admin`
  or `write`; every comment is included, marked as context.
- **Gemini CLI** ([run-gemini-cli](https://github.com/google-github-actions/run-gemini-cli)):
  OWNER, MEMBER or COLLABORATOR for `/fix`; its `/approve` approves the
  agent's plan, not anyone's words.
- **OpenHands** ([github-issue-to-pr](https://github.com/OpenHands/extensions/tree/main/skills/github-issue-to-pr)),
  **feiskyer/github-fix-issue** ([skill](https://github.com/feiskyer/claude-code-settings/tree/main/skills/github-fix-issue)),
  **ccpm** ([automazeio/ccpm](https://github.com/automazeio/ccpm)): no
  distinction between authors.
- **Claude Code itself** ([permission modes](https://code.claude.com/docs/en/permission-modes)):
  writes to `.claude/`, `.mcp.json` and `.git` are asked about or
  classified, whatever the task's origin; `CLAUDE.md` and `.github/` are not
  on that list.

## GitHub's mechanics the design needs

Verified against [docs.github.com](https://docs.github.com) and live calls on
2026-11-14.

- **`pull_request_target`** runs the workflow file as it is on the
  **default** branch (since 2025-12-08), for forks and branches alike
  ([changelog](https://github.blog/changelog/2025-11-07-actions-pull_request_target-and-environment-branch-protections-changes/));
  its check suite attaches to the pull request's head commit (checked live)
  and satisfies required checks; its token can write even for forks.
  `actions/checkout` refuses a fork's code in it unless told otherwise
  ([changelog](https://github.blog/changelog/2026-06-18-safer-pull_request_target-defaults-for-github-actions-checkout)).
- **Blocked by default in public repositories from 2026-12-02**, with
  brownouts on 2026-11-09 and 2026-11-16 (16:00–18:00 UTC), unless an
  Actions event policy allows it; a policy can be scoped to one workflow
  file (`conditions.workflow_path`), by the UI (Settings → Actions →
  Policies) or `POST /repos/{owner}/{repo}/actions/policies`
  ([changelog](https://github.blog/changelog/2026-11-02-pull_request_target-blocked-by-default-december-2),
  [how-to](https://docs.github.com/en/actions/how-tos/administer/control-workflow-execution),
  [REST](https://docs.github.com/en/rest/actions/policies)). The pages name
  no other event as blocked.
- **`issue_comment`** runs attach to the default branch's commit, so a gate
  re-run by a comment must post a **commit status** on the head commit; a
  status whose context matches a required check satisfies it, and a
  ruleset can require it from the GitHub Actions app (`integration_id`
  15368) ([rules REST](https://docs.github.com/en/rest/repos/rules)).
  Unverified: that a status posted with `GITHUB_TOKEN` is attributed to
  15368 (high confidence; check after the first one).
- **Spoofing:** a fork's own `pull_request` workflow can create a *check
  run* with the same name; where a check run and a status share a name both
  must pass, so the gate's own `pending` status closes that window (our
  inference from the docs, not tested). Statuses are overwritten by the
  latest of a context; only push access or a workflow with
  `statuses: write` can post one.
- **Roles:** `GET /repos/{o}/{r}/collaborators/{u}/permission` answers the
  owner of a personal repository `admin`/`admin`, a non-collaborator
  `read`/`read` (200, not 404), a missing user 404 (live). Personal
  repositories have no maintain role: collaborators are `write`.
- **Edits:** GraphQL `lastEditedAt` and `editor` are null until the body is
  edited, and a title change does not set them (a `RenamedTitleEvent`
  instead); anyone with write access can edit others' comments and issue
  bodies ([roles](https://docs.github.com/en/organizations/managing-user-access-to-your-organizations-repositories/managing-repository-roles/repository-roles-for-an-organization)).
- **Rulesets** are changed by admins only, never by `GITHUB_TOKEN`;
  `bypass_mode` `always` (direct pushes too), `pull_request` (merging only)
  or `exempt`; `RepositoryRole` admin is id 5 (undocumented). An author
  cannot approve their own pull request, so a sole owner's code-owner
  review is met only by bypass. A pull request is reviewed against the
  base branch's `CODEOWNERS`.

<!-- trust-gate smoke test: this branch is thrown away -->
