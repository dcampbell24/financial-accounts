.PHONY: debian enable-git-hooks

debian:
	/bin/sh debian.sh

enable-git-hooks:
	git config --local core.hooksPath .githooks/