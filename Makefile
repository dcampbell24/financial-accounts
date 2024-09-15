.PHONY: debian enable-git-hooks

debian:
	/usr/bin/sh debian.sh

enable-git-hooks:
	git config --local core.hooksPath .githooks/