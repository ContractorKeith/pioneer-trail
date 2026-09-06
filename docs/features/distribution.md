# Distribution

Release binaries are built natively for Linux x86_64 and arm64, macOS Apple
Silicon and Intel, and Windows x86_64. The release workflow always runs its
build, lint, test, content-lint, and headless smoke checks; it publishes only
for a pushed version tag.

## Release steps

1. Set the workspace version in `Cargo.toml` and confirm the release branch is
   green.
2. Create and push the matching annotated tag:

   ```bash
   git tag -a v0.1.0 -m "Pioneer Trail v0.1.0"
   git push origin v0.1.0
   ```

3. Confirm the GitHub Actions release job completed and download a released
   archive to smoke-test the binary.

The workflow uses the repository `GITHUB_TOKEN` with `contents: write` to
create the GitHub Release. Manually dispatching a workflow requires a GitHub
account authorized to run repository Actions, but needs no additional release
secret and cannot publish. The repository currently has no crates.io publish
token configured, so crate publication is blocked until a
`CARGO_REGISTRY_TOKEN` secret with publish permission is added.

When that token exists and the previous package has resolved on crates.io,
publish dependent crates in this order:

```bash
cargo publish -p pioneer-sim
cargo publish -p pioneer-data
cargo publish -p pioneer-trail
```

## Homebrew

The intended distribution path is a source formula in the ContractorKeith
Homebrew tap. It builds the path workspace from source and does not depend on
crates.io publication. A formula and tap publication remain TODO until the
GitHub release process has been exercised; do not claim the tap is live before
that release is published.
