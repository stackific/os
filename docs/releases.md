# Releases

The images reach GitHub Releases in two ways. Both use the recipes in `release/justfile`. Builds and boot tests run on the self-hosted runners. GitHub-hosted runners open the draft release and publish it.

## A release

Run the `release` workflow by hand (Actions, release, Run workflow) with a semver version without the `v`, such as `0.1.0`.

1. `just release create <version> <sha>` refuses a version that is not semver, or one whose release or tag `v<version>` already exists. It then opens a draft release at the workflow's commit with generated notes. A version with a suffix (`0.1.0-rc.1`) becomes a prerelease.
2. Each image job runs `just <base> iso <version>` and `just <base> test <version>`, then `just release upload <version> <base> <arch>`. That uploads the ISO, its package manifest and `<iso>.sha256` to the draft.
3. Once every image job passed, `just release publish <version>` joins the `.sha256` files into `SHA256SUMS`, attaches it and publishes the release. A failed job leaves the draft unpublished: delete it on GitHub before running the same version again.

Today the release holds the x86_64 images from the operator's PC and the Arch aarch64 image from the Mac's runner VM, unsigned. Fedora aarch64 and signing join when they exist.

## Test releases

After each merge to `main`, the `images` workflow builds and boot-tests every image. `just release pr-version <sha>` finds the pull request that merged the commit, and the images become the prerelease `v0.0.0-pr.<n>`, titled "Test build for #<n>". A rerun replaces it. A commit that no pull request merged is built and tested but not published.

Test releases are not releases: they are unsigned, and a later cleanup may delete them.
