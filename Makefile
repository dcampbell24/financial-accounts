enable-git-hooks:
	git config --local core.hooksPath .githooks/

install-audit:
	cargo install cargo-audit

install-msrv:
	cargo install cargo-msrv

install-vet:
	cargo install cargo-vet