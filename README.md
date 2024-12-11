# bif

`bif` is a CLI tool and library for working with [BIF](https://developer.roku.com/en-ca/docs/developer-program/media-playback/trick-mode/bif-file-creation.md#file-format) files.

## Usage

Using the compiled binary:

```
$ bif --help
bif 0.1.0

USAGE:
    bif <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    decode
    encode
    help      Prints this message or the help of the given subcommand(s)
```

### Decode

```rust
use bif;
use std::path::PathBuf;

let b = bif::decode(PathBuf::from("index.bif"));
// Check how many images are in the BIF: b.total_images

// Save all images in the `output/` directory (making the output directory if it doesn't exist)
bif::extract_images(b, PathBuf::from("output/"));
```

via the CLI:

```
$ bif decode index.bif output
BIF Version: 0
Number of images: 700
Framewise Separation: 1000ms"
Generating images...
Finished.
```

Saves the images stored within `index.bif` to the directory `output/` where images will have filenames in the format `frame_%020d.jpg` and the number is the frame timestamp in milliseconds.

### Encode

```rust
use bif;
use std::path::PathBuf;

let mut jpegs = Vec::<PathBuf>::new();
// Build a vector of PathBufs to store 100 images
jpegs.push(PathBuf::from("./images/frame_00.jpg"));
// ...
jpegs.push(PathBuf::from("./images/frame_99.jpg"));
// Sort the images sequentially
jpegs.sort();
let b = bif::encode(jpegs, PathBuf::from("index.bif"), 1, 1000);
// Expect 100 images to have been stored in the resulting BIF
assert_eq!(b.total_images, 100, "Not all images were stored.");
```

via the CLI:

```
$ bif encode images index.bif
BIF Version: 0
Number of images: 1400
Timestamp Interval: 1
Framewise Separation: 1000ms"
Finished.
```

Saves the JPG images from `images/` into a BIF file named `index.bif`.

---

**NOTE**

The BIF specification allows for storing arbitrary timestamp values for each frame image and in any order. This encoder does not currently support that level of specificity and instead assumes that all frame images are equidistant from one another starting at 0 and incrementing by 1 for each subsequent frame by default.

The `--ti` encoding option is provided to adjust the timestamp interval along with the `--fs` encoding option to alter the multiplier.

If you wanted to represent each frame being 2 seconds apart the following commands would all be equivalent:

- `bif encode output index.bif --ti 2`
- `bif encode output index.bif --ti 2 --fs 1000`
- `bif encode output index.bif --ti 1 --fs 2000`
- `bif encode output index.bif --ti 4 --fs 500`

## License

Copyright © 2024 Brendon Walsh.

Licensed under MIT (see the file [LICENSE](./LICENSE)).
