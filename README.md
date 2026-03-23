# steve

content management tool for my iPod.

## Config

`steve` reads TOML config from:

- `$XDG_CONFIG_HOME/steve/config.toml` (if `XDG_CONFIG_HOME` is set)
- `~/.config/steve/config.toml` (fallback)

Example:

```toml
rss-urls = [
  "https://example.com/feed.xml",
  { url = "https://example.com/private.xml", max-episodes = "all" }
]
episodes-dir = "/home/my_user/episodes"
max-episodes = 20
```

Options:

- `rss-urls`: array of feed entries.
- Feed entry as string: uses top-level `max-episodes`.
- Feed entry as inline table: can override `max-episodes` per feed.
- `episodes-dir`: output directory.
- `max-episodes`: positive integer or `"all"` (default is `20`).

Notes:

- Download concurrency is fixed to two workers per CPU core.
- When `max-episodes` is numeric, old episode files not in the latest set are pruned.

