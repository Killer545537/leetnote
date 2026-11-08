# leetnote

Generate an Obsidian note from a LeetCode problem URL.

Given a problem URL, `leetnote` fetches the problem from LeetCode, converts the statement to Markdown, and writes a ready-to-fill note into a directory of your choice, such as your Obsidian vault.

## What it does

1. Extracts the problem slug from the URL (query parameters and a trailing `/description/` are ignored).
2. Fetches the problem's ID, title, difficulty, tags and statement from LeetCode.
3. Converts the HTML statement to Markdown.
4. Writes `<Problem Title>.md` to the output directory, creating the directory if it does not exist.

The note contains:

- YAML frontmatter: `created`, `number`, `tags`, `difficulty`, `url`, `type: problem` and `status: todo`.
- The problem statement in a `> [!problem]` callout.
- Empty `Observations`, `Key Insight` and `Algorithm` sections for your own notes.

`leetnote` never overwrites an existing note. If a file with the same name is already there, it exits with an error and leaves the file untouched.

Characters that are not allowed in filenames (such as `/ \ : ? * " < > |`) are replaced with spaces.

## Usage

```sh
leetnote <URL> --output-dir <DIR>
```

| Argument                 | Description                                  |
| ------------------------ | -------------------------------------------- |
| `<URL>`                  | A `https://leetcode.com/problems/...` URL    |
| `-o`, `--output-dir <DIR>` | Directory where the note will be created   |

### Example

```sh
cargo run -- \
  "https://leetcode.com/problems/assign-cookies/" \
  --output-dir "$HOME/Documents/Obsidian Vault/Leetcode"
```

```
Note created: /Users/you/Documents/Obsidian Vault/Leetcode/Assign Cookies.md
```

Quote the URL (it often contains `&`). Quote the directory if it contains spaces, but don't also escape the spaces with backslashes, or the backslashes become part of the path.

### Accepted URLs

- `https://leetcode.com/problems/two-sum/`
- `https://leetcode.com/problems/two-sum/description/`
- `https://leetcode.com/problems/two-sum/?envType=study-plan`

Only `https` URLs on `leetcode.com` with a `/problems/<slug>` path are accepted.

## Build

```sh
cargo build --release
```

The binary is written to `target/release/leetnote`.
