psitool
=======

psitool is a 100% free and open-source toolset built in Rust to help you practice Remote Viewing and download and
maintain target pools, as well as pick a training image.

The main tool is `psi-target-pool`. Assuming you have a valid config at `~/.psitool.yaml` (see Config Format below),
and have downloaded target images (see `psi-wm-downloader` utility help section below), then you can use the main
binary to generate a 100% blind remote-viewing target.

    Usage: psi-target-pool [OPTIONS]

    Options:
      -v, --verbose                        verbose logging (debug logs)
      -q, --quiet                          quiet logging (warn+ logs)
      -f...                                how much to frontload, none by default (pass -f for 1 level of frontloading, -ff for 2, -fff for 3...)
      -s, --skip-open                      dont open the target after
      -c, --config <CONFIG>                the config with the target pools [default: ~/.psitool.yaml]
      -p, --pools <POOLS>                  the named target pool to read from (included unless excluded via label)
      -i, --include-label <INCLUDE_LABEL>  the target pools to read from, including this label
      -x, --exclude-label <EXCLUDE_LABEL>  the target pools to read from, EXCLUDING this label
      -h, --help                           Print help
      -V, --version                        Print version

Basically, if no options are passed, it will use every pool you defined in the config, with every JPEG/JPG/SVG/TARGET
in each of their directories.

The output will look like this when run from the command-line, and then you can press Enter to see the target:

    $ psi-target-pool
    [2025-09-27T09:48:43Z] INFO: including pool 'training' because no options passed (all pools)
    [2025-09-27T09:48:43Z] INFO: including pool 'personal' because no options passed (all pools)
    [2025-09-27T09:48:43Z] INFO: found 2 target pools to match
    [2025-09-27T09:48:43Z] INFO: pool ~/Documents/rv_pools/train: 221 jpgs
    [2025-09-27T09:48:43Z] INFO: pool ~/Documents/rv_pools/personal_pool: 0 jpgs
    [2025-09-27T09:48:43Z] INFO: Total jpgs: 221
    [2025-09-27T09:48:43Z] INFO: Chose rvuid R-2DTH-GZW5-W9FMX29F6HJ52Q8N9C
    Target: R-2DTH-GZW5-W9FMX29F6HJ52Q8N9C
    Press ENTER to see target.
    Remote viewer, begin.

Notice the RVUID provided, `R-2DTH-GZW5-W9FMX29F6HJ52Q8N9C`. This is generated via the uuid5 function, which is
non-random and generated using the actual bytes of the jpeg. The same exact file (bit by bit) will generate the same
"RVUID", or Remote Viewing Unique Identifier.

This is like a UUID, except it uses base-32 digits (all digits, most uppercase letters except I/L/O/U since they
can be confused with 1/0), and it splits the 128-bits into 3 sections with a static `R-` prefix.

Basically, as a remote viewer it should be enough to just write `R-2DTH-GZW5` as above, but that is just 40-bits. You
can always save it to file, or I will write a utility in the future to find the image by RVUID (short and long).

Right now, it will wait until you press enter so you can practice, then see the target image. I will add some helpers
to work with it to also output the description from the YAML files that are coupled with each downloaded JPEG from
wikimedia.

You can _always_ create your own jpg/jpeg/target files and throw them in a directory. This is made so you can maintain
your own private target pools as well.

It also supports text files with the `.target` extension, so you can write out your own target like so:

    echo "The workers burying the body of Alexander the Great, where and when he was buried." > ~/Documents/pool_dir/atg.target

For example:

    $ psi-target-pool -i me -q
    Target: R-THZP-Q1S2-3S9EKA24H6CK7DN9F4
    Remote viewer, begin viewing.
    Press ENTER when complete.

    Path: ~/Documents/rv_pools/personal_pool/bar.baz.target
    Target Text:
    San Francisco, the golden gate bridge.


In this case, it will use the bytes of the text file just as it would the JPG so you still get a normal RVUID that is
specific to the _exact_ target text.

Config Format
-------------

The config file is by default `~/.psitool.yaml` and should look something like this:

    target_pools:
      training:
        path: ~/Documents/rv_pools/train
        labels: [train, wiki]
        wiki:
          default_limit: 2000
          queries:
            - query: animal
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
            - query: statue
            - query: historical
              limit: 5000
            - query: event
      training_with_frontloading:
        path: ~/Documents/rv_pools/train_with_fl
        labels: [train, frontload, wiki]
        wiki:
          default_limit: 1000
          queries:
            - query: historical event
              frontloading: ["historical event"]
            - query: natural landscape
              frontloading: ["natural landscape", "more specific frontloading", "even more specific"]
      personal:
        path: ~/Documents/rv_pools/personal_pool
        labels: [me]

This defines all your target pools, keyed by their name.

Above, you see three pools. One is the `personal` pool with label `me`, and has a path to a document directory (which
will be created if it does not exist when being downloaded to).

The `training` pool is interesting here because it provides a specification about what it will download from wikimedia
if you want to generate all the target pool images to use for training purposes. Each query pattern will ask for 2000
images, unless you put in a custom limit like under "historical" in the above example, which will get 5000 instead of
2000.

It will query for each of those, using the `limit` provided.

Notice the other pool `training_with_frontloading`, where each query has a list of frontloading phrases.

These are completely optional, and even if you save frontloading data, you have to request psi-target-pool to output
it when remote viewing. By default, it shows none. If you want the first item (most generic frontloading) from the
list, pass `-f`, which might be "historical event" or "natural landscape" as seen above. If you want the next more
specific frontloading, pass `-ff`, and then `-fff`, and so on. It will only output what you ask for, if it has it.

What this does is just download the data, and save the metadata in the associated `$path.jpg.yaml` under a key
`frontloading`. You can edit this for any target you have and customize the YAML. This is just a helper for
downloading training datasets.

Be aware that the query might not always give you an exact example of what you frontload. For example, if you put
"biological" as frontloading for the animal query, this is not exactly correct for the "animal" query which sometimes
gives you images like an amulet shaped like a wolf. I would suggest using something more generic, or customize the
yaml files manually.

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
