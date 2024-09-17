.PHONY: debian enable-git-hooks

debian:
	sh debian.sh

enable-git-hooks:
	git config --local core.hooksPath .githooks/
