#! /bin/sh

pandoc\
    --variable=title:financial-accounts\
    --variable=section:1\
    --variable=date:2024-07-14\
    --standalone --to=man financial-accounts.1.dj --output=financial-accounts.1
gzip --no-name --best financial-accounts.1
cargo deb
rm financial-accounts.1.gz