# alpha-srom

Code for decoding the SROM from an AlphaStation. Only supports Alpha 21164 encoded images at the moment.

Requires `alpha-linux-gnu-objdump` to be installed, and an SROM image file.

Use `expand.py` to unpack `am27c010_image_alphastation500.bin` (a byte-wise SROM file) into eight bitstreams called `srom_0.bin` to `srom_7.bin`.

Use `cargo run -- ./srom_0.bin ./srom_0.decoded ./srom_0.asm` to decode.

See https://thejpster.org.uk/blog/blog-2025-03-30 for more details.
