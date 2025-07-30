# Makefile for mdbook-bib version management

.PHONY: help update-version update-cargo update-doc release

# Default target
help:
	@echo "Available targets:"
	@echo "  update-version VERSION=x.y.z  - Update version in both Cargo.toml and doc.yml"
	@echo "  update-cargo VERSION=x.y.z    - Update version only in Cargo.toml"
	@echo "  update-doc VERSION=x.y.z      - Update version only in doc.yml"
	@echo "  release VERSION=x.y.z         - Complete release process (update, commit, tag, push)"
	@echo "  help                          - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  make update-version VERSION=0.0.8"
	@echo "  make update-cargo VERSION=0.0.8"
	@echo "  make update-doc VERSION=0.0.8"
	@echo "  make release VERSION=0.0.8"

# Main target to update version in both files
update-version: update-cargo update-doc
	@echo "Version updated to $(VERSION) in both Cargo.toml and doc.yml"

# Update version in Cargo.toml
update-cargo:
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION parameter is required. Use: make update-cargo VERSION=x.y.z"; \
		exit 1; \
	fi
	@echo "Updating version to $(VERSION) in Cargo.toml..."
	@sed -i.bak 's/^version = ".*"/version = "$(VERSION)"/' Cargo.toml
	@rm -f Cargo.toml.bak
	@echo "✓ Cargo.toml updated"

# Update version in doc.yml
update-doc:
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION parameter is required. Use: make update-doc VERSION=x.y.z"; \
		exit 1; \
	fi
	@echo "Updating MDBOOK_BIB_VERSION to $(VERSION) in .github/workflows/doc.yml..."
	@sed -i.bak 's/^\([[:space:]]*MDBOOK_BIB_VERSION: \).*/\1$(VERSION)/' .github/workflows/doc.yml
	@rm -f .github/workflows/doc.yml.bak
	@echo "✓ doc.yml updated"

# Complete release process: update version, commit changes, create and push tag
release: update-version
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION parameter is required. Use: make release VERSION=x.y.z"; \
		exit 1; \
	fi
	@echo "Starting release process for version $(VERSION)..."
	@echo "Adding changes to git..."
	@git add Cargo.toml .github/workflows/doc.yml
	@echo "Committing changes..."
	@git commit -m "Prepare for release v$(VERSION)"
	@echo "Creating tag v$(VERSION)..."
	@git tag -a v$(VERSION) -m "Version v$(VERSION)"
	@echo "Pushing commit and tag to origin..."
	@git push origin master
	@git push origin v$(VERSION)
	@echo "✓ Release v$(VERSION) completed successfully!" 
