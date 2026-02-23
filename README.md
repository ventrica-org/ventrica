# Ventrica

Wannabe atomic and undecided architecture package manager for macOS. Also written in rust, and a beautiful swift ui.

## Building

Building is straight forward, to build all binaries, you can just do

```sh
cargo build --workspace --release
```

`ventricad` will need to be running as root in the background, or else the GUI & CLI will be unable to do proper actions.

`/ventrica` must also exist, since this is primarily for mac you will be needing to create a new volume

these need a reboot after
```
diskutil apfs addVolume disk3 APFS Ventrica
cat ventrica >> /etc/synthetic.conf
```
automount vol to `/ventrica`
```
$ cat /etc/fstab
#
# Warning - this file should only be modified with vifs(8)
#
# Failure to do so is unsupported and may be destructive.
#
UUID=<VOLUME_UDID> /ventrica apfs nosuid,noatime,owners
```