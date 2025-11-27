# ------------------------------------------------------------------------------
# Grove Makefile
# ------------------------------------------------------------------------------

.PHONY: help build web wasm demo assets pdf test clean rust engine-run docker ci

help:
	@echo "Makefile targets:"
	@echo "  make build        # build everything (rust + web + wasm + assets)"
	@echo "  make web          # build Next.js app"
	@echo "  make wasm         # build Rust/WASM modules"
	@echo "  make demo         # build browser demos (wasm + assets)"
	@echo "  make assets       # regenerate thumbnails/rothko frames"
	@echo "  make pdf          # generate final submission PDF"
	@echo "  make test         # run unit tests"
	@echo "  make clean        # clean build artifacts"
	@echo "  make rust         # build all Rust crates (workspace)"
	@echo "  make engine-run   # run engine-server (release or fallback)"
	@echo "  make ci           # CI pipeline (rust + wasm + web)"

# ------------------------------------------------------------------------------

build: rust web wasm assets

# ------------------------------------------------------------------------------
# Web / Next.js layer
# ------------------------------------------------------------------------------

web:
	@if [ -f package.json ]; then \
		echo "Building Next.js app..."; \
		npm ci --silent || true; \
		npm run build --if-present; \
	else \
		echo "No package.json found — skipping web build."; \
	fi

# ------------------------------------------------------------------------------
# WASM build
# ------------------------------------------------------------------------------

wasm:
	@echo "Building WASM modules..."
	@./scripts/build_wasm.sh

# ------------------------------------------------------------------------------
# Demos
# ------------------------------------------------------------------------------

demo: wasm
	@echo "Packaging demos..."
	@cp -r demos/builder-game/dist public/demos/builder-game 2>/dev/null || true

# ------------------------------------------------------------------------------
# Assets
# ------------------------------------------------------------------------------

assets:
	@echo "Generating assets..."
	python3 tools/rothko_gen.py --out public/assets/visuals/rothko_frame_1080.png

# ------------------------------------------------------------------------------
# PDF generation
# ------------------------------------------------------------------------------

pdf:
	@echo "Generating PDF..."
	python3 scripts/generate_pdf.py

# ------------------------------------------------------------------------------
# Rust / Engine
# ------------------------------------------------------------------------------

rust:
	@echo "Building Rust workspace..."
	cd engine && cargo build --workspace --release

engine-run: rust
	@echo "Starting engine-server..."
	./engine/target/release/engine-server || \
	cargo run --manifest-path engine/engine-server/Cargo.toml --release

# ------------------------------------------------------------------------------
# Tests
# ------------------------------------------------------------------------------

test:
	@echo "Running tests..."
	cargo test --workspace

# ------------------------------------------------------------------------------
# Cleanup
# ------------------------------------------------------------------------------

clean:
	@echo "Cleaning build artifacts..."
	rm -rf public/demos || true
	rm -rf demos/*/dist || true

# ------------------------------------------------------------------------------
# CI aggregation target
# ------------------------------------------------------------------------------

ci: rust wasm web
	@echo "CI build complete."
