set shell := ["bash", "-c"]

# Build the WASM package and prepare assets
build:
    wasm-pack build --target web
    mkdir -p public
    cp index.html sample.html public/
    cp -r pkg public/

# Run the project locally
dev: build
    npx wrangler dev

# Deploy to Cloudflare
deploy: build
    npx wrangler deploy

# Clean build artifacts
clean:
    rm -rf pkg public target
