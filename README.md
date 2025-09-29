psitool
=======

psitool is a 100% free and open-source toolset built in Rust to help you practice Remote Viewing and download and
maintain target pools, as well as pick a training image.

The main tool is `psi-target-pool`. Assuming you have a valid config at `~/.psitool.yaml` (see Config Format below),
and have downloaded target images (see `psi-wm-downloader` utility help section below), then you can use the main
binary to generate a 100% blind remote-viewing target.

Completed targets are stored in `~/.psitool_completed_targets.yaml` by default, but you can point at any path.
Comment them out if you want to reuse those targets, or pass `--reuse-targets` as an argument to reuse all of them.

    The 100% free and open-source Remote Viewing toolset.

    Usage: psi-target-pool [OPTIONS]

    Options:
      -v, --verbose                        verbose logging (debug logs)
      -q, --quiet                          quiet logging (warn+ logs)
      -r, --reuse-targets                  reuse all targets, even if they're already completed
      -f...                                how much to frontload, none by default (pass -f for 1 level of frontloading, -ff for 2, -fff for 3...)
      -s, --skip-open                      dont open the target after
      -c, --config <CONFIG>                the config with the target pools [default: ~/.psitool.yaml]
      -C, --completed <COMPLETED>          the yaml config with a list of completed targets (used to cache what you RV'd already) [default: ~/.psitool_completed_targets.yaml]
      -p, --pools <POOLS>                  the named target pool to read from (included unless excluded via label)
      -i, --include-label <INCLUDE_LABEL>  the target pools to read from, including this label
      -x, --exclude-label <EXCLUDE_LABEL>  the target pools to read from, EXCLUDING this label
      -h, --help                           Print help
      -V, --version                        Print version

Basically, if no options are passed, it will use every pool you defined in the config, with every JPEG/JPG/SVG/TARGET
in each of their directories. It will process whatever you have in `~/.psitool_completed_targets.yaml` and skip the
pre-used targets, unless you pass `-r` or `--reuse-targets`

The output will look like this when run from the command-line, and then you can press Enter to see the target.

(Use `--quiet` or `-q` to suppress info logs)

    $ psi-target-pool

    [2025-09-28T09:43:20Z] INFO: including pool 'personal' because no options passed (all pools)
    [2025-09-28T09:43:20Z] INFO: including pool 'train' because no options passed (all pools)
    [2025-09-28T09:43:20Z] INFO: found 2 target pools to match
    [2025-09-28T09:43:20Z] INFO: pool ~/Documents/rv_pools/personal_pool: 2 targets
    [2025-09-28T09:43:31Z] INFO: pool ~/Documents/rv_pools/train: 1423 targets
    [2025-09-28T09:43:31Z] INFO: Total targets: 1425
    [2025-09-28T09:43:52Z] INFO: Chose rvuid R-DB56-29KT-XS94S59E2HMFYZW9GC

    Target: R-DB56-29KT-XS94S59E2HMFYZW9GC
    Remote viewer, begin viewing.
    Press ENTER when complete.

    ... <pressed enter> ...

    Path: ~/Documents/rv_pools/train/Sculptureum.jpg
    YAML meta: ~/Documents/rv_pools/train/Sculptureum.jpg.yaml
    Query: sculpture
    Description: "Sculptureum"
    Datetime: "31 December 2022"
    License: CC BY-SA 4.0
    Was it a hit ([y]es, [n]o, otherwise not saved/recorded)? y
    Score out of 100 (0 to 100 or otherwise not saved/recorded)? 25
    Any notes? Press enter to end (or blank to not save anything): i thought it was an arm being held up, but it was an elephant trunk

    [2025-09-28T05:45:42Z] INFO: Succesfully wrote 1 completed targets to ~/.psitool_completed_targets.yaml

**Note**: In the future, I will update it to cache parsed images, but for now, every single run it re-hashes every
target you are selecting so this can take a bit longer than it needs to, since it's literally reading every target
you are randomly selecting from.

Notice the RVUID provided, `R-DB56-29KT-XS94S59E2HMFYZW9GC`. This is generated via the uuid5 function, which is
non-random and generated using the actual bytes of the target file. The same exact file (bit by bit) will generate
the same "RVUID", or Remote Viewing Unique Identifier.

This is like a UUID, except it uses base-32 digits (all digits, most uppercase letters except I/L/O/U since they
can be confused with 1/0), and it splits the 128-bits into 3 sections with a static `R-` prefix.

Basically, as a remote viewer it should be enough to just write `R-2DTH-GZW5` as above, but that is just 40-bits and
ot the full RVUID. However, all completed targets are saved to the completed config file.

I will write a utility in the future to find the image by RVUID (via short or long).

You can _always_ create your own jpg/jpeg/target files and throw them in a directory. This is made so you can maintain
your own private target pools as well.

It also supports text files with the `.target` extension, so you can write out your own target like so:

    echo "The workers burying the body of Alexander the Great, where and when he was buried." > ~/Documents/pool_dir/atg.target

For example:

$ psi-target-pool -i me -q
    Target: R-P14S-9E46-JXEE1030TDSB5Y6KJM
    Remote viewer, begin viewing.
    Press ENTER when complete.

    ...<pressed enter>...

    Path: ~/Documents/rv_pools/personal_pool/foo.bar.target
    Target Text:
    The Mona Lisa painting, where it is as of 9/2025.

    Was it a hit ([y]es, [n]o, otherwise not saved/recorded)? y
    Score out of 100 (0 to 100 or otherwise not saved/recorded)? 55
    Any notes? Press enter to end (or blank to not save anything):

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

psi-rvuid-gen
-------------

This utility generates RVUID hashes from paths.

    Usage: psi-rvuid-gen [OPTIONS] [PATHS]...

    Arguments:
      [PATHS]...  paths to hash and determine rvuid

    Options:
      -v, --verbose  verbose logging (debug logs)
      -q, --quiet    quiet logging (warn+ logs)
      -h, --help     Print help
      -V, --version  Print version

Example:

    $ psi-rvuid-gen ./README.md
    ./README.md = R-4NJX-Z0BV-E9AAHEDRNBNXT8RRC4

    $ psi-rvuid-gen ./README.md src/*
    ./README.md = R-4NJX-Z0BV-E9AAHEDRNBNXT8RRC4
    src/config.rs = R-DG4D-A9VQ-XSFX3AQKW95HKNC32M
    src/lib.rs = R-3344-2KR0-ZXEEDBTPPS9RE9MV5R
    src/logger.rs = R-Z562-F0JF-NHEVXA5SAYG9G2V928
    src/rvuid.rs = R-PHAJ-P5S5-5197V7YXRRWM4TBA30
    src/target.rs = R-3BVX-6WZ6-TSB070P33PHWWJ0A2W

psi-rvuid-find
--------------

This utility will look for provided RVUIDs (but they must be the full 128-bit RVUID, not the short version).

    Usage: psi-rvuid-find [OPTIONS] [RVUIDS]...

    Arguments:
      [RVUIDS]...  the RVUIDs to look for

    Options:
      -v, --verbose          verbose logging (debug logs)
      -q, --quiet            quiet logging (warn+ logs)
      -D, --find-dupes       keep searching even if you already found every RVUID (find potential dupes)
      -c, --config <CONFIG>  the config with the target pools (this is where it will look for the RVUID) [default: ~/.psitool.yaml]
      -h, --help             Print help
      -V, --version          Print version

Example:

    $ psi-rvuid-find R-BRHR-XGP6-E5BENEWQXDBEXBKN4W R-ZANQ-CJK4-JD969E5TFGATJCX3VW

    R-BRHR-XGP6-E5BENEWQXDBEXBKN4W found at: ~/Documents/rv_pools/train/2014_Prowincja_Sjunik,_Klasztor_Tatew_(19).jpg
    R-ZANQ-CJK4-JD969E5TFGATJCX3VW found at: ~/Documents/rv_pools/train/2013-Aerial-Mount_of_Olives.jpg

Now works with shortened 40-bit formats (like R-GWVD-CYBT , so you don't have to write everything down):

    $ psi-rvuid-find R-GWVD-CYBT R-HN40-R5YJ-PNF8V2M9Y37FKPPEW4
    R-GWVD-CYBT-2D9DS3D0FP0A93QH54 found at: ~/Documents/rv_pools/personal_pool/test2.target
    R-HN40-R5YJ-PNF8V2M9Y37FKPPEW4 found at: ~/Documents/rv_pools/personal_pool/test1.target

    $ echo $?
    0

    $ psi-rvuid-find R-GWVD-CYBT R-HN40-R5YJ-PNF8V2M9Y37FKPPEW0
    R-GWVD-CYBT-2D9DS3D0FP0A93QH54 found at: ~/Documents/rv_pools/personal_pool/test2.target
    Error: missing: [Rvuid { uuid: 8d480c17-d2b5-5e8d-8a89-f0cef9dacee0, rvuid: "R-HN40-R5YJ-PNF8V2M9Y37FKPPEW0", missing_bits: false, prefix40: 606799140818 }]

    $ echo $?
    1

    $ psi-rvuid-find R-GWVD-CYBT R-HN40-R5YJ
    R-GWVD-CYBT-2D9DS3D0FP0A93QH54 found at: ~/Documents/rv_pools/personal_pool/test2.target
    R-HN40-R5YJ-PNF8V2M9Y37FKPPEW4 found at: ~/Documents/rv_pools/personal_pool/test1.target

    $ echo $?
    0


Roadmap
-------

I'm open to suggestions, so feel free to contact me at psitool #at# protonmail #dot# com

The next obvious feature I'm planning is Associated Remote Viewing functionality.
