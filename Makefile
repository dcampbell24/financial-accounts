enable-git-hooks:
	git config --local core.hooksPath .githooks/

install-audit:
	cargo install cargo-audit