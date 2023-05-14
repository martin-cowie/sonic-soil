# sonic-soil
A barest bones CLI for Sonos. 'Sonic Soil' is an anagram of 'Sonos CLI'

## Building

`cargo build --release` to build locally

`cargo install --path .` to build and install into your Rust path.

## Use

`sonic-soil list` to discover and list your Sonos zones.

`sonic-soil join zone0 zone1 [zone2]` to join Sonos zones into a group.

