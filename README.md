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
