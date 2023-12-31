#! /bin/sh

pandoc --standalone --to=man fin-stat.1.md --output=fin-stat.1
gzip --no-name --best fin-stat.1
cargo deb
rm fin-stat.1.gz