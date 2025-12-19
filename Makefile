# Guardian OS Build System with Supabase Integration
SHELL := /bin/bash
.PHONY: all clean assets debs repo iso sync help bump-version bump-minor bump-major version-sync

# Configuration
GPG_KEYID ?= guardian@gameguardian.ai
AWS_PROFILE ?= default
CF_DISTRIBUTION_ID ?= 
REPO_S3_BUCKET ?= apt.gameguardian.ai
ISO_VERSION := $(shell cat VERSION 2>/dev/null || echo "1.0.0")
ISO_NAME = guardian-os-$(ISO_VERSION)-amd64.iso

# Supabase Configuration (for build-time reference)
SUPABASE_URL = https://xzxjwuzwltoapifcyzww.supabase.co
GUARDIAN_API_BASE = $(SUPABASE_URL)/functions/v1

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
NC := \033[0m

help:
	@echo "Guardian OS Build System"
	@echo "========================"
	@echo "  make assets  - Fetch brand assets"
	@echo "  make debs    - Build all .deb packages"
	@echo "  make repo    - Create/update APT repository"
	@echo "  make iso     - Build complete ISO image"
	@echo "  make sync    - Sync repo to S3"
	@echo "  make clean   - Clean build artifacts"

all: version-sync iso

version-sync:
	@echo -e "$(GREEN)Syncing version $(ISO_VERSION) to all files...$(NC)"
	@scripts/update-version.sh

assets: version-sync
	@echo -e "$(GREEN)Fetching brand assets...$(NC)"
	@scripts/fetch-assets.sh

debs: assets
	@echo -e "$(GREEN)Building Debian packages...$(NC)"
	@scripts/build-debs.sh

repo: debs
	@echo -e "$(GREEN)Building APT repository...$(NC)"
	@GPG_KEYID=$(GPG_KEYID) scripts/build-repo.sh

iso: repo
	@echo -e "$(GREEN)Building ISO image...$(NC)"
	@scripts/iso-build.sh
	@echo -e "$(GREEN)ISO built: $(ISO_NAME)$(NC)"

sync: repo
	@echo -e "$(GREEN)Syncing to S3...$(NC)"
	@AWS_PROFILE=$(AWS_PROFILE) REPO_S3_BUCKET=$(REPO_S3_BUCKET) \
		CF_DISTRIBUTION_ID=$(CF_DISTRIBUTION_ID) scripts/sync-s3.sh

clean:
	@echo -e "$(YELLOW)Cleaning build artifacts...$(NC)"
	@rm -rf iso/chroot iso/binary iso/.build
	@rm -rf packages/*/debian/files packages/*/debian/*.debhelper*
	@rm -rf packages/*/debian/*.substvars packages/*/debian/guardian-*/
	@rm -rf repo/db repo/dists repo/pool
	@find . -name "*.deb" -delete
	@find . -name "*.buildinfo" -delete
	@find . -name "*.changes" -delete

# Version bumping - increments patch version
bump-version:
	@CURRENT=$$(cat VERSION); \
	MAJOR=$$(echo $$CURRENT | cut -d. -f1); \
	MINOR=$$(echo $$CURRENT | cut -d. -f2); \
	PATCH=$$(echo $$CURRENT | cut -d. -f3); \
	NEW_PATCH=$$((PATCH + 1)); \
	NEW_VERSION="$$MAJOR.$$MINOR.$$NEW_PATCH"; \
	echo "$$NEW_VERSION" > VERSION; \
	echo -e "$(GREEN)Version bumped: $$CURRENT -> $$NEW_VERSION$(NC)"

bump-minor:
	@CURRENT=$$(cat VERSION); \
	MAJOR=$$(echo $$CURRENT | cut -d. -f1); \
	MINOR=$$(echo $$CURRENT | cut -d. -f2); \
	NEW_MINOR=$$((MINOR + 1)); \
	NEW_VERSION="$$MAJOR.$$NEW_MINOR.0"; \
	echo "$$NEW_VERSION" > VERSION; \
	echo -e "$(GREEN)Version bumped: $$CURRENT -> $$NEW_VERSION$(NC)"

bump-major:
	@CURRENT=$$(cat VERSION); \
	MAJOR=$$(echo $$CURRENT | cut -d. -f1); \
	NEW_MAJOR=$$((MAJOR + 1)); \
	NEW_VERSION="$$NEW_MAJOR.0.0"; \
	echo "$$NEW_VERSION" > VERSION; \
	echo -e "$(GREEN)Version bumped: $$CURRENT -> $$NEW_VERSION$(NC)"
