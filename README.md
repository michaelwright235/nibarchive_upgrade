# NIB Archive Upgrade

Convert Apple's NIB Archive `.nib` to Cocoa Keyed Archive (NSKeyedArchive) `.plist`.

NIB Archives are used by UIKit (since iOS 6.0) and AppKit for building GUIs.
In the past UIKit and AppKit actually used Cocoa Keyed Archives to
store this data. But later Apple decided to replace it with NIB Archive.

NIB Archive is actually very similar to a regular Keyed Archive:
it also contains objects, values, references, etc. So it's possible to "upgrade"
NIB archives back to Keyed Archives.

The conversion is lossless *most of the times*. A NIB Archive's class may contain
`fallback classes` that are not represented in Keyed Archives and cannot be
included in the resulting file.
One example that I was able to find is the `NSColor` fallback class of the
`UIColor` class.

## Why?

NIB archive is a quite niche format, and so is Keyed Archive. However the latter
is actually better documented and has more tools to work with it.

## Use

The following example converts the `foo.nib` NIB Archive to the `foo.plist`
Cocoa Keyed Archive.

### Using a Rust Library

```rust
use nibarchive_upgrade::upgrade;
use nibarchive_upgrade::nibarchive::NIBArchive;
use plist::Value;

let archive: NIBArchive = NIBArchive::from_file("./foo.nib")?;
let plist: Value = upgrade(&archive);
plist.to_file_binary("./foo.plist")?;
```

### Using an executable

```bash
./nibarchive_upgrade foo.nib foo.plist
```
