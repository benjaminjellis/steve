# steve

content management tool for my iPod.

## Config

`steve` reads EDN config from:

- `$XDG_CONFIG_HOME/steve/config.edn` (if `XDG_CONFIG_HOME` is set)
- `~/.config/steve/config.edn` (fallback)

Example:

```clojure
{:rss-urls ["https://example.com/feed.xml"
            {:url "https://example.com/private.xml"
             :max-episodes :all
             :number true}]
 :episodes-dir "/home/my_user/episodes"
 :max-episodes 20}
```

Options:

- `rss-urls`: vector of feed entries.
- Feed entry as string: uses top-level `max-episodes`.
- Feed entry as map: can override `max-episodes` per feed.
- `number`: optional per-feed boolean. `true` prefixes episode filenames with a number counting from the oldest entry in the full feed, zero-padded for sorting (e.g. `01 - Episode.mp3`). Assumes the feed lists newest episodes first: the last entry gets 1 and the first gets the full feed count. Missing, `nil`, or `false` leaves names unchanged. Numbers are assigned before filtering or applying `max-episodes`, and can change if older entries are removed from the feed.
- `episodes-dir`: output directory.
- `max-episodes`: positive integer at the top level; feed overrides also accept `:all`.

Notes:

- Download concurrency is fixed to two workers per CPU core.
- When `max-episodes` is numeric, old episode files not in the latest set are pruned.
