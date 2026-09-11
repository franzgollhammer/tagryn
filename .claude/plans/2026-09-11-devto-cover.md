# Host the dev.to cover and article images

## Objective

Publish the approved Tagryn cover image to a focused GitHub branch and reference publicly accessible repository images from the dev.to Markdown draft. Keep the article unpublished.

## Approaches

1. **Commit the images and use commit-pinned raw URLs (selected).** This keeps the assets reviewable in the repository and makes the article references independent of later branch changes. It requires an image commit before the article can reference that commit.
2. **Commit the images and use branch-based raw URLs.** This allows a single commit and easy replacement, but later branch changes or deletion can change or break the article images.
3. **Upload images to a GitHub release.** This provides direct download URLs, but adds unrelated promotional assets to a software release and does not place the cover in the repository source tree.

## Implementation

1. Create `docs/devto-cover` in a new worktree from `origin/develop`.
2. Add the approved AI-generated cover, based on Tagryn's official app icon, as `docs/images/tagryn-devto-cover.png` without changing its pixels.
3. Commit the cover and record its commit ID.
4. Set `cover_image` to the raw cover URL pinned to that commit. Restore the existing synthetic workspace screenshot inside the article, using the same commit ID, and correct its caption.
5. Store the article as `docs/posts/introducing-tagryn.md` and synchronize the existing local dev.to draft.
6. Commit the article and push the focused branch. Do not merge or modify `main`.

## Verification

- Compare the repository cover with the approved source using SHA-256 and inspect the PNG dimensions.
- Check Markdown formatting, frontmatter, image references, and `published: false`.
- Run `git diff --check` and inspect the staged changes before committing.
- Fetch both public image URLs after pushing and compare their SHA-256 hashes with the repository files.
- Verify the remote branch head and clean worktree state.
- Follow the documentation-only verification guidance in `CONTRIBUTING.md`; no application code changes or local full-suite runs are needed. Use the repository's existing `[skip ci]` convention for these documentation commits.

## Unresolved questions

None. The user authorized pushing the cover and updating the Markdown. Merging remains outside this task and requires separate permission.
