# Contributing to Tagryn

Tagryn is an offline metadata workbench maintained by [Franz Gollhammer](https://github.com/franzgollhammer). Contributions made with AI are welcome. The person submitting a contribution is responsible for understanding it, checking it, and following up on review.

## Find a useful change

Start with the [roadmap](docs/ROADMAP.md) and existing [issues](https://github.com/franzgollhammer/tagryn/issues). Reproduction steps, translations, documentation, accessibility findings, and small bug fixes are valuable contributions. Discuss substantial features or changes to writing behavior in an issue before implementing them. The maintainer decides product direction and which changes are accepted.

Use English for shared documentation and pull requests where possible. German reports are welcome. Be respectful, assume good intent, and focus criticism on the work. Harassment, personal attacks, and repeated spam may result in removal or a block.

## Set up a development checkout

Fork the repository, clone your fork, and create a branch in a separate worktree. The integration branch is `develop`.

```sh
git clone https://github.com/YOUR-USERNAME/tagryn.git
cd tagryn
git remote add upstream https://github.com/franzgollhammer/tagryn.git
git fetch upstream
git worktree add -b fix/describe-the-change ../tagryn-my-change upstream/develop
cd ../tagryn-my-change
npm ci
npm run runtime
npm run icons
npm run licenses
npm run desktop
```

Install the platform dependencies described in the [README](README.md#build-from-source) first. The first runtime build downloads pinned packages and compiles private Perl on macOS/Linux. Start with disposable media copies; the desktop app performs real filesystem writes.

Read [AGENTS.md](AGENTS.md) when using an agent. Before changing file writes, restore, sidecars, or the engine, read [SAFETY.md](docs/SAFETY.md), [FORMATS.md](docs/FORMATS.md), and the relevant [architecture](docs/ARCHITECTURE.md) section. Those detailed documents are currently in German; English translations are welcome.

## Verify your change

Run `npm run check` for TypeScript changes and `cargo fmt --manifest-path src-tauri/Cargo.toml --check` for Rust changes. Run only the specific local tests relevant to your change. For example, after `npm run fixtures`:

```sh
# Choose the relevant file; these are examples, not a local full-suite recipe.
node --test tests/import.test.mjs
cargo test --manifest-path src-tauri/Cargo.toml --locked --test sidecars
```

The **complete test suite runs only in GitHub Actions**, including on contributors' machines with Docker. Push your branch to your fork with Actions enabled, or open a pull request. [Verify Tagryn](.github/workflows/verify.yml) runs the Rust integrity tests, Node tests, Playwright checks, runtime verification, and platform bundles. Fork pull requests may require maintainer approval before GitHub starts the workflow. CI bundles are unsigned development artifacts, not official installer releases.

Add a regression test when fixing behavior that could damage files or lose metadata. Use generated fixtures and temporary directories. Never check private images, metadata exports, local databases, `.tagryn-backups`, credentials, or runtime build output into Git. For UI changes, attach a screenshot using synthetic files and inspect it for personal paths or metadata first.

## Submit a pull request

Target `develop`. Keep the change focused and describe the problem, resulting behavior, verification actually performed, and remaining limits. Link the related issue. For AI-assisted work, a short note about the tools used and your own verification is enough; private prompt transcripts are unnecessary. Do not present an agent's claimed test result as a test you ran.

Retain original attribution and license notices when reusing code or assets. Submit only material you have the right to contribute. Contributions are offered under the project's [MIT license](LICENSE); third-party components retain their own terms. There is no separate contributor agreement.

Maintainer review and a successful applicable CI run are required before acceptance. Agents must obtain explicit maintainer permission before any merge. `main` is not used: do not commit, push, or merge into it. Maintainers may request a smaller change or decline work that does not fit the [project principles](docs/PRINCIPLES.md).
