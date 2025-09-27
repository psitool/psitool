psitool
=======

Config Format
-------------

The config file is by default `~/.psitool.yaml` and should look like this:

    target_pools:
      training:
        path: ~/Documents/rv_pools/train
        labels: [train, wiki]
        wiki:
          default_limit: 2000
          queries:
            - query: animal
              limit: 500
            - query: building
            - query: landscape
            - query: vehicle
            - query: portrait
            - query: sculpture
            - query: painting
            - query: object
            - query: artifact
            - query: tool
            - query: music
            - query: food
              limit: 500
            - query: statue
            - query: historical
            - query: event
      training2:
        path: ~/Documents/rv_pools/train2
        labels: [train]
      personal:
        path: ~/Documents/rv_pools/personal_pool
        labels: [me]

This defines all your target pools, keyed by their name.

Above, you see two pools. One is the `personal` pool with label `me`, and has a path to a document directory (which
will be created if it does not exist when being downloaded to).

The `training` pool is interesting here because it provides a specification about what it will download from wikimedia
if you want to generate all the target pool images to use for training purposes.

It will queyr for each of those, using the `limit` provided such as `500` for the `animal` query, otherwise if limit
is not specified it will use `2000`.

psi-wm-downloader
-----------------

This handy tool will take a target pool name and download all sorts of RV training data for yourself.
Each image will be downloaded _sequentially_, which is relatively slow but also should be a one-time process,
and thus nicer to the wikimedia servers.

You need a valid config at `~/.psitool.yaml` (or passed in as `-c|--config`) that specifies the queries to use to
fill the specified target pool.

For example, for the above you would run `psi-wm-downloader training`

    Usage: psi-wm-downloader [OPTIONS] <POOL>

    Arguments:
      <POOL>  the target pool to download for

    Options:
      -v, --verbose          verbose logging (debug logs)
          --free-only        only download CC0, CC-BY, and PUBLIC DOMAIN so they can be rebundled
      -c, --config <CONFIG>  the config with the target pools [default: ~/.psitool.yaml]
      -l, --limit <LIMIT>    override wiki default_limit
      -h, --help             Print help
      -V, --version          Print version

**Users are responsible for reusing images under their correct license terms.**

By default, it will download whatever wikimedia gives you. However, you can pass `--free-only` to skip everything
that isn't CC0, CC-BY (not CC-BY-SA/NC/ND), and PUBLIC DOMAIN.

CC-BY requires you (I believe, I am not a lawyer) to redistribute if you give credit to the original artist/owner. The
YAML files that are associated with each JPG (same path, but + ".yaml") include that License data provided from
wikimedia. You _should_ be able to use that, _I think_. Again, I'm not a lawyer, but this is my best effort to allow
you to download free content and share it. I solely provide this downloader, none of the images.
