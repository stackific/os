# Releases

The images reach GitHub Releases in two ways. Both use the recipes in `release/justfile`. Builds and boot tests run on the self-hosted runners. GitHub-hosted runners open the draft release and publish it.

## A release

Run the `release` workflow by hand (Actions, release, Run workflow) with a semver version without the `v`, such as `0.1.0`.

1. `just release create <version> <sha>` refuses a version that is not semver, or one whose release or tag `v<version>` already exists. It then opens a draft release at the workflow's commit, titled "Stackific OS <version>". A version with a suffix (`0.1.0-rc.1`) becomes a prerelease. The notes are under [Release notes](#release-notes).
2. Each image job runs `just <base> iso <version>` and `just <base> test <version>`, then `just release upload <version> <base> <arch>`. That uploads the ISO, its package manifest and `<iso>.sha256` to the draft. GitHub takes release files up to 2 GiB, so an ISO over that goes to Cloudflare R2 instead: the pinned rclone (`just rclone install`) copies it to `os/releases/v<version>/<name>.iso` in the bucket, served at `https://cdn.stackific.com/os/releases/v<version>/<name>.iso`, and the release gets `<name>.iso.url`, that URL on one line. `SHA256SUMS` lists it like the others. The upload steps of `images.yml` and `release.yml` get three environment variables: the repository secrets `R2_ACCESS_KEY_ID` and `R2_SECRET_ACCESS_KEY`, and the repository variable `R2_ENDPOINT`, the bucket's S3 API URL as Cloudflare's bucket settings show it (`https://<account>.r2.cloudflarestorage.com/<bucket>`). rclone gets the keys only through its environment. A test build's rerun overwrites its object.
3. Once every image job passed, `just release publish <version>` joins the `.sha256` files into `SHA256SUMS`, attaches it and publishes the release. A failed job leaves the draft unpublished: delete it on GitHub before running the same version again.

Today the release holds all four images, unsigned: the x86_64 images from the operator's PC and the aarch64 images from the Mac's runner VM. The Mac has one runner, so its two jobs run one after the other. Signing joins when it exists.

### Cutting v0.1.0

v0.1.0 is the first release: a live preview of the four images, with no installer.

1. Merge task 011, and wait for the `images` run of the merge commit to pass. `gh release view v0.0.0-pr.<n> --json body --jq .body` shows the test line, then `release/notes.md`.
2. Run the `release` workflow on `main` with version `0.1.0`, and wait for all six jobs: create, four images, publish.
3. `gh release view v0.1.0 --json isDraft,isPrerelease,assets` shows a published release that is not a prerelease, with 13 assets: four ISOs, four `.packages.txt` files, four `.iso.sha256` files and `SHA256SUMS`. `gh release list` marks it Latest.
4. In an empty folder, `gh release download v0.1.0 -p '*.iso' -p SHA256SUMS`, then `sha256sum -c SHA256SUMS`, prints four lines ending in `: OK`.

## Release notes

`release/notes.md` opens the notes of every release and test build. A test build puts its own line first ("Test build of the images after pull request #<n> merged: …"). The file says what a release is today (a live preview, unsigned), how to check a download, and what each ISO is built from. A task that changes any of those, such as the installer or signing, updates it.

Under it, a release gets GitHub's generated notes (the merged pull requests), starting at the newest full release (`--notes-start-tag`). Left alone, GitHub starts them at the newest tag, which is a test build (`v0.0.0-pr.<n>`), and lists only the last pull request. The first release has no full release to start from, so it gets no generated notes; `release/notes.md` links the prompt trail instead. Preview generated notes without creating anything: `gh api -X POST repos/stackific/os/releases/generate-notes -f tag_name=v<version> -f target_commitish=main -f previous_tag_name=v<last>`.

### Base labels

The notes, the site and the README name what each image is built from only in these words, which Arch's and Fedora's trademark rules allow (checked 2026-11-05):

| ISO | Label | Rule |
| --- | --- | --- |
| `-arch-x86_64.iso` | Based on Arch Linux | Arch Linux trademark policy (2021-04-18) allows "based on Arch Linux"; names beginning with ARCH need permission |
| `-arch-aarch64.iso` | Based on Arch Linux ARM | Built from Arch Linux ARM, a separate project |
| `-fedora-x86_64.iso`, `-fedora-aarch64.iso` | Fedora Remix | Fedora's trademark guidelines (2024-12-05) allow only the Fedora Remix mark for distributions derived from Fedora; any other wording with Fedora's name, "based on" included, needs the Fedora Council's permission |

Where the Fedora Remix mark appears, two more things go with it:

- Fedora's notice: the Fedora Remix images contain modified Fedora® packages and other software, are not provided or supported by the Fedora Project, and official, unmodified Fedora releases are at https://fedoraproject.org/.
- Red Hat's trademark line: "Fedora and Fedora Remix are trademarks or registered trademarks of Red Hat, Inc., or its subsidiaries in the United States and other countries." The site's footer (`site/src/components/starlight/Footer.astro`) carries it on every docs page.

The words "Fedora" and "Remix" are never split, a mark always modifies a noun ("Fedora Remix images"), and the first standalone "Fedora" on a page carries ®. No Arch or Fedora logo appears anywhere.

## Test releases

The `images` workflow builds and boot-tests the images on every push to `main` (a merged pull request), the x86_64 images on the PC and the aarch64 images on the Mac, and when started by hand (Actions tab, or `gh workflow run images --ref main`). A push builds only the images whose files it changed; a run by hand builds all four. `just release changed <before> <sha>` decides, image by image, from the paths each one is built and tested from (an Arch-only change skips both Fedora images, an `arch/iso/packages.aarch64` change skips x86_64, a docs-only change builds nothing); when `<before>` is not in the history, every image is built. `just release pr-version <sha>` finds the pull request that merged the commit, and the images built become the prerelease `v0.0.0-pr.<n>`, titled "Test build for #<n>". A rerun replaces it. A commit that no pull request merged is built and tested but not published, and a push that changed no image makes no test release. A run that a newer push cancels never builds its own changes: rerun `images` for that commit when they matter.

Test releases are not releases: they are unsigned, and a later cleanup may delete them.
