# BW-RS

Open Source Implementation of Starcraft Brood War in Rust. Inspired by [OpenBW](https://github.com/OpenBW/openbw).

## Setup

You must own a legal copy of Starcraft have the assets `STARDAT.MPQ`, `BROODAT.MPQ`, `patch_rt.mpq` available (not remastered).

Copy these assets into the `bw_core/assets` folder.

Download the [Starcraft SSACTI map pack](https://sscaitournament.com/files/sscai_map_pack.zip) and unzip them into the maps folder.

Modify [bw_config.ron](./bw_core/config/bw_config.ron) with the map name you want.

Execute `cargo run --release`.
