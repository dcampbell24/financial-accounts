#! /bin/sh

cargo build --release
./target/release/financial-accounts --man

gzip --no-name --best financial-accounts.1

cargo deb

rm financial-accounts.1.gz
