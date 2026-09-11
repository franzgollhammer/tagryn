# Tagryn open source release

## Objective

Publish Tagryn under `franzgollhammer/tagryn` as an open source project that welcomes contributions made with AI. Omarchy is a reference for a clear product direction, accessible onboarding, explicit agent guidance, and maintainer-led review. This does not imply a relationship with Omarchy.

## Three release approaches

1. **Public source alpha (recommended).** Publish the working source, an open source license, contributor documentation, issue forms, and full GitHub Actions verification. This gives contributors a useful starting point quickly. The cost is requiring a development toolchain; signed installers remain a later milestone.
2. **Binary preview launch.** Complete the operating-system matrix, signing, notarization, clean-machine installation checks, and versioned installers before launch. This lowers the barrier for end users but depends on signing credentials and more platform testing.
3. **Website-led launch.** Add a separate product site, demo video, public roadmap, and installer release together. This gives stronger product communication but increases scope and delays community access.

## Selected implementation

Use the source-alpha path. The existing app remains version 0.1.0; the first public prerelease is identified as `v0.1.0-alpha.1`. MIT is proposed because it matches the reference project and permits broad reuse. Apache-2.0 and GPL-3.0 were offered as alternatives. An unanswered license preference is not approval to publish a particular license; prepare the candidate and obtain a decision before public licensing.

Keep development in a new worktree at `~/dev/worktrees/tagryn/feat-open-source-release/`. The original `feat/desktop-app` worktree contains uncommitted work and remains untouched. The repository has no history or remote. Bootstrap a `develop` default branch from the reviewed initial source snapshot; do not create, commit to, push to, or merge into `main`. No merge is required for an initial repository.

## Work

1. Inspect the local application, workflow, dependencies, generated material, and original validation notes. Research the supplied Omarchy pages using primary sources and record the comparison.
2. Prepare an English README with a German usage guide, product principles, contributor guidance for humans and agents, a compact AGENTS.md, issue forms, PR template, security-reporting instructions, and a practical roadmap.
3. Add consistent project license metadata while retaining third-party license terms. Exclude generated runtime, local reports, fixture media, private profiles, and unsigned local packages from publication. Review each promotional image before inclusion.
4. Verify all documentation links, metadata, workflow references, whitespace, and the staged source snapshot. Run the complete test suite only through GitHub Actions. Use a private repository for prepublication CI if needed.
5. Once licensing is decided and the public snapshot is concrete, publish the repository and a source prerelease with honest verification status and known limitations. Enable Issues, Discussions, and private vulnerability reporting. Verify the public repository, license detection, default branch, release target, and workflow outcome.

## Completion criteria

The public repository and source prerelease exist under the requested account, the license is chosen, contributor entry points are functional, and validation results are reported accurately. Production installer readiness is not claimed. If public licensing remains undecided, preserve a reviewable candidate and explicitly request that final decision.

## Unresolved questions

- Which license should apply: MIT (recommended), Apache-2.0, or GPL-3.0?
- Which signing identities will be used for later macOS and Windows installer releases? This does not block the source alpha.
- Is a separate Tagryn domain and website desired after the repository launch? This does not block the source alpha.
