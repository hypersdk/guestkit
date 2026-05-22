PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
CARGO ?= cargo
BINARY = guestkit

.PHONY: all build release install uninstall clean check test fmt clippy selftest \
	deploy deploy-remote deploy-remote-quick deploy-remote-preflight deploy-remote-verify deploy-remote-uninstall deploy-remote-fleet

all: build

build:
	$(CARGO) build

release:
	$(CARGO) build --release

install: release
	install -Dm755 target/release/$(BINARY) $(DESTDIR)$(BINDIR)/$(BINARY)

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(BINARY)

clean:
	$(CARGO) clean

check: test clippy

test:
	$(CARGO) test

fmt:
	$(CARGO) fmt

clippy:
	$(CARGO) clippy -- -D warnings

selftest:
	bash scripts/selftest.sh

deploy: deploy-remote ## Alias for deploy-remote

deploy-remote: ## Full remote deploy: make deploy-remote H=ip U=root
	@test -n "$(H)" || { echo "  ❌ H (host) is required"; exit 1; }
	bash scripts/deploy-remote.sh $(H) $(or $(U),root) $(PASS) --key $(ARGS)

deploy-remote-quick: ## Quick remote deploy: make deploy-remote-quick H=ip U=root
	@test -n "$(H)" || { echo "  ❌ H is required"; exit 1; }
	bash scripts/deploy-remote.sh $(H) $(or $(U),root) $(PASS) --key --quick $(ARGS)

deploy-remote-preflight: ## SSH preflight only: make deploy-remote-preflight H=ip
	@test -n "$(H)" || { echo "  ❌ H is required"; exit 1; }
	bash scripts/deploy-remote.sh $(H) $(or $(U),root) --key --preflight-only

deploy-remote-verify: ## Remote selftest only: make deploy-remote-verify H=ip
	@test -n "$(H)" || { echo "  ❌ H is required"; exit 1; }
	bash scripts/deploy-remote.sh $(H) $(or $(U),root) --key --verify-only

deploy-remote-uninstall: ## Remove guestkit from host: make deploy-remote-uninstall H=ip
	@test -n "$(H)" || { echo "  ❌ H is required"; exit 1; }
	bash scripts/deploy-remote.sh $(H) $(or $(U),root) --key --uninstall

deploy-remote-fleet: ## Fleet deploy: make deploy-remote-fleet FILE=hosts.txt
	@test -n "$(FILE)" || { echo "  ❌ FILE is required"; exit 1; }
	bash scripts/deploy-remote.sh --fleet $(FILE)
