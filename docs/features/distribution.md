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
create the GitHub Release. No credential is needed for a manual workflow run,
which builds artifacts but cannot publish. Publishing crates later requires a
crates.io token with publish permission; keep it in a GitHub Actions secret
rather than a repository file.

## Homebrew

The intended distribution path is a source formula in the ContractorKeith
Homebrew tap. A formula and tap publication remain TODO until the crate and
GitHub release process have been exercised; do not claim the tap is live before
that release is published.
