PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
CARGO ?= cargo
BINARY = guestkit

.PHONY: all build release install uninstall clean check test fmt clippy selftest deploy

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

deploy:
	@echo "Usage: make deploy HOST=<ip> [USER=root] [PASS=<password>] [ARGS=--quick]"
	@test -n "$(HOST)" || { echo "  ❌ HOST is required"; exit 1; }
	bash scripts/deploy-remote.sh $(HOST) $(or $(USER),root) $(PASS) $(ARGS)
