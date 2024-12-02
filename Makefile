.PHONY: enable-git-hooks
enable-git-hooks:
	git config --local core.hooksPath .githooks/

.PHONY: install-audit
install-audit:
	cargo install cargo-audit

.PHONY: install-msrv
install-msrv:
	cargo install cargo-msrv

.PHONY: install-vet
install-vet:
	cargo install cargo-vet