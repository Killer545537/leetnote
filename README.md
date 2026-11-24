# leetnote

Generate an Obsidian note from a LeetCode problem URL.

Given a problem URL, `leetnote` fetches the problem from LeetCode, converts the statement to Markdown, and writes a ready-to-fill note into a directory of your choice, such as your Obsidian vault.

## What it does

1. Extracts the problem slug from the URL (query parameters and a trailing `/description/` are ignored).
2. Fetches the problem's ID, title, difficulty, tags and statement from LeetCode.
3. Converts the HTML statement to Markdown.
4. Converts math such as `0 <= nums[i] <= 10^{9}` to LaTeX (`$0 \leq \mathtt{nums[i]} \leq 10^{9}$`). Apple's on-device model, through the `fm` CLI (macOS 27+), only decides which spans are math; the conversion itself is done by fixed rules, so numbers can't be altered. If `fm` is missing or fails, the note is still written with the math as LeetCode wrote it, and a warning is printed.
5. Writes `<Problem Title>.md` to the output directory, creating the directory if it does not exist.

The note contains:

- YAML frontmatter: `created`, `number`, `tags`, `difficulty`, `url`, `type: problem` and `status: todo`.
- The problem statement in a `> [!problem]` callout.
- Empty `Observations`, `Key Insight` and `Algorithm` sections for your own notes.

`leetnote` never overwrites an existing note. If a file with the same name is already there, it exits with an error and leaves the file untouched.

Characters that are not allowed in filenames (such as `/ \ : ? * " < > |`) are replaced with spaces.

## Requirements

- **To run:** macOS on Apple silicon for the prebuilt binary. Other platforms can build from source.
- **For LaTeX math (optional):** Apple's `fm` CLI, which ships with macOS 27 at `/usr/bin/fm`. It runs the on-device model, so it needs no network or API key, but Apple Intelligence must be available on the Mac.
  - Accept the model license once: `sudo fm license`. Check it with `fm license --status`.
  - Check that the model is ready: `fm models` should show `✓ system`.
  - Without `fm`, or with `--no-normalize`, notes are still written; the math just stays as LeetCode wrote it.

## Install

Download `leetnote-<version>-aarch64-apple-darwin.tar.gz` from the [latest release](https://github.com/Killer545537/leetnote/releases/latest), then:

```sh
tar -xzf leetnote-*-aarch64-apple-darwin.tar.gz
sudo mv leetnote /usr/local/bin/
```

The binary isn't notarized, so if macOS refuses to open it, clear the quarantine flag: `xattr -c /usr/local/bin/leetnote`.

Or build from source (see [Build](#build)).

## Usage

```sh
leetnote <URL> --output-dir <DIR>
leetnote --daily --output-dir <DIR>
```

| Argument                 | Description                                  |
| ------------------------ | -------------------------------------------- |
| `<URL>`                  | A `https://leetcode.com/problems/...` URL    |
| `--daily`                | Use today's daily challenge instead of a URL |
| `-o`, `--output-dir <DIR>` | Directory where the note will be created. Defaults to `$LEETNOTE_DIR` |
| `--no-normalize`         | Skip the LaTeX conversion (no `fm` call)     |

### Example

```sh
cargo run -- \
  "https://leetcode.com/problems/assign-cookies/" \
  --output-dir "$HOME/Documents/Obsidian Vault/Leetcode"
```

```
Note created: /Users/you/Documents/Obsidian Vault/Leetcode/Assign Cookies.md
```

To skip `-o` every time, set a default directory in your shell profile. An explicit `-o` still takes precedence:

```sh
export LEETNOTE_DIR="$HOME/Documents/Obsidian Vault/Leetcode"
leetnote "https://leetcode.com/problems/assign-cookies/"
leetnote --daily
```

Quote the URL (it often contains `&`). Quote the directory if it contains spaces, but don't also escape the spaces with backslashes, or the backslashes become part of the path.

### Accepted URLs

- `https://leetcode.com/problems/two-sum/`
- `https://leetcode.com/problems/two-sum/description/`
- `https://leetcode.com/problems/two-sum/?envType=study-plan`

Only `https` URLs on `leetcode.com` with a `/problems/<slug>` path are accepted.

Premium problems are rejected with an error: LeetCode doesn't return their statement without a subscription.

## Build

```sh
cargo build --release
```

The binary is written to `target/release/leetnote`.
