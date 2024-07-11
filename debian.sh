#! /bin/sh

pandoc --standalone --to=man financial-accounts.1.md --output=financial-accounts.1
gzip --no-name --best financial-accounts.1
cargo deb
rm financial-accounts.1.gz