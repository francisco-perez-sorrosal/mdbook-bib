# Makefile for mdbook-bib version management

.PHONY: help update-version update-cargo update-doc release show-version check-release update-lockfile update-changelog

# Get current version from Cargo.toml
CURRENT_VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

# Auto-increment patch version if VERSION not specified
# Splits x.y.z and increments z
ifndef VERSION
  VERSION_MAJOR := $(word 1,$(subst ., ,$(CURRENT_VERSION)))
  VERSION_MINOR := $(word 2,$(subst ., ,$(CURRENT_VERSION)))
  VERSION_PATCH := $(word 3,$(subst ., ,$(CURRENT_VERSION)))
  NEXT_PATCH := $(shell echo $$(($(VERSION_PATCH) + 1)))
  VERSION := $(VERSION_MAJOR).$(VERSION_MINOR).$(NEXT_PATCH)
  AUTO_VERSION := 1
endif

# Dry-run support: set DRY_RUN=1 to simulate commands
ifdef DRY_RUN
  RUN := @echo "[DRY-RUN]"
  DRY_RUN_MSG := (DRY-RUN)
else
  RUN := @
  DRY_RUN_MSG :=
endif

# Default target
help:
	@echo "mdbook-bib Release Management"
	@echo "=============================="
	@echo ""
	@echo "Current version: $(CURRENT_VERSION)"
	@echo "Next version:    $(VERSION) (auto-incremented patch)"
	@echo ""
	@echo "Targets:"
	@echo "  show-version                  - Show current and next version"
	@echo "  update-version                - Update version in Cargo.toml and doc.yml"
	@echo "  update-cargo                  - Update version only in Cargo.toml"
	@echo "  update-doc                    - Update version only in doc.yml"
	@echo "  update-lockfile               - Regenerate Cargo.lock after version update"
	@echo "  update-changelog              - Generate CHANGELOG.md using git-cliff"
	@echo "  check-release                 - Validate release preconditions"
	@echo "  release                       - Complete release (update, commit, tag, push)"
	@echo ""
	@echo "Options:"
	@echo "  VERSION=x.y.z                 - Specify version (default: auto-increment patch)"
	@echo "  DRY_RUN=1                     - Simulate without making changes"
	@echo ""
	@echo "Examples:"
	@echo "  make release                      # Release $(VERSION) (auto)"
	@echo "  make release VERSION=1.0.0        # Release specific version"
	@echo "  make release DRY_RUN=1            # Simulate release $(VERSION)"
	@echo "  make update-version DRY_RUN=1     # Simulate version update"

# Show current and computed next version
show-version:
	@echo "Current version: $(CURRENT_VERSION)"
	@echo "Next version:    $(VERSION)"
ifdef AUTO_VERSION
	@echo "(auto-incremented from $(CURRENT_VERSION))"
endif

# Main target to update version in both files
update-version: update-cargo update-doc
	@echo "$(DRY_RUN_MSG)✓ Version updated to $(VERSION) in both Cargo.toml and doc.yml"

# Update version in Cargo.toml
update-cargo:
	@echo "$(DRY_RUN_MSG)Updating version to $(VERSION) in Cargo.toml..."
ifdef DRY_RUN
	@echo "[DRY-RUN] sed -i.bak 's/^version = \".*\"/version = \"$(VERSION)\"/' Cargo.toml"
	@echo "[DRY-RUN] Current: version = \"$(CURRENT_VERSION)\""
	@echo "[DRY-RUN] New:     version = \"$(VERSION)\""
else
	@sed -i.bak 's/^version = ".*"/version = "$(VERSION)"/' Cargo.toml
	@rm -f Cargo.toml.bak
endif
	@echo "$(DRY_RUN_MSG)✓ Cargo.toml updated"

# Update version in doc.yml
update-doc:
	@echo "$(DRY_RUN_MSG)Updating MDBOOK_BIB_VERSION to $(VERSION) in doc.yml..."
ifdef DRY_RUN
	@echo "[DRY-RUN] sed -i.bak 's/MDBOOK_BIB_VERSION: .*/MDBOOK_BIB_VERSION: $(VERSION)/' .github/workflows/doc.yml"
	@echo "[DRY-RUN] Current: $$(grep 'MDBOOK_BIB_VERSION:' .github/workflows/doc.yml | head -1)"
else
	@sed -i.bak 's/^\([[:space:]]*MDBOOK_BIB_VERSION: \).*/\1$(VERSION)/' .github/workflows/doc.yml
	@rm -f .github/workflows/doc.yml.bak
endif
	@echo "$(DRY_RUN_MSG)✓ doc.yml updated"

# Regenerate Cargo.lock after version update
update-lockfile:
	@echo "$(DRY_RUN_MSG)Regenerating Cargo.lock..."
ifdef DRY_RUN
	@echo "[DRY-RUN] cargo update --workspace"
else
	@cargo update --workspace
endif
	@echo "$(DRY_RUN_MSG)✓ Cargo.lock updated"

# Generate CHANGELOG using git-cliff
update-changelog:
	@echo "$(DRY_RUN_MSG)Generating CHANGELOG.md for v$(VERSION)..."
ifdef DRY_RUN
	@echo "[DRY-RUN] git-cliff --tag v$(VERSION) -o CHANGELOG.md"
else
	@if ! command -v git-cliff >/dev/null 2>&1; then \
		echo "Error: git-cliff not found. Install with: cargo install git-cliff"; \
		exit 1; \
	fi
	@git-cliff --tag v$(VERSION) -o CHANGELOG.md
endif
	@echo "$(DRY_RUN_MSG)✓ CHANGELOG.md updated"

# Check release preconditions
check-release:
ifndef DRY_RUN
	@echo "Checking release preconditions..."
	@if ! echo "$(VERSION)" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$$'; then \
		echo "Error: Invalid version format '$(VERSION)'. Expected: x.y.z"; \
		exit 1; \
	fi
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "Error: Working directory has uncommitted changes."; \
		echo "Please commit or stash changes before releasing."; \
		git status --short; \
		exit 1; \
	fi
	@echo "✓ All preconditions passed"
else
	@echo "[DRY-RUN] Would check: version format, clean working directory"
endif

# Complete release process
release: check-release update-version update-lockfile update-changelog
	@echo ""
	@echo "$(DRY_RUN_MSG)Starting release process for version $(VERSION)..."
	@echo ""
	@echo "$(DRY_RUN_MSG)[1/4] Staging changes..."
	$(RUN) git add Cargo.toml Cargo.lock .github/workflows/doc.yml CHANGELOG.md
	@echo "$(DRY_RUN_MSG)[2/4] Committing..."
	$(RUN) git commit -m "Prepare for release v$(VERSION)"
	@echo "$(DRY_RUN_MSG)[3/4] Creating tag v$(VERSION)..."
	$(RUN) git tag -a v$(VERSION) -m "Version v$(VERSION)"
	@echo "$(DRY_RUN_MSG)[4/4] Pushing to origin (atomic)..."
	$(RUN) git push origin master v$(VERSION)
	@echo ""
	@echo "$(DRY_RUN_MSG)✓ Release v$(VERSION) completed!"
	@echo ""
	@echo "$(DRY_RUN_MSG)GitHub Actions will now run sequentially:"
	@echo "$(DRY_RUN_MSG)  1. Release  → Build binaries (Linux/Win/macOS)"
	@echo "$(DRY_RUN_MSG)  2. Publish  → Publish to crates.io (if builds pass)"
	@echo "$(DRY_RUN_MSG)  3. Docs     → Deploy to GitHub Pages (if publish passes)"
ifdef DRY_RUN
	@echo ""
	@echo "Run 'make release' or 'make release VERSION=$(VERSION)' to execute for real."
endif
