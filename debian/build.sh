#! /bin/sh

pandoc\
    --variable=title:financial-accounts\
    --variable=section:1\
    --variable=date:2024-07-14\
    --standalone --to=man debian/financial-accounts.1.dj --output=debian/financial-accounts.1

gzip --no-name --best debian/financial-accounts.1
pandoc --standalone --to=plain README.md --output=debian/README.txt

cargo deb

rm debian/financial-accounts.1.gz
rm debian/README.txt