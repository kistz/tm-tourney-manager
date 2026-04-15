#!/bin/bash
# Publish module as "server-manager"
spacetime publish --break-clients --delete-data=on-conflict -y -p tm-server-manager tmservers-wqk3g 

# Generate Rust and TS client APIs
spacetime generate --yes --lang rust --out-dir tm-server-manager-api-rs/src/generated --module-path tm-server-manager
spacetime generate --yes --lang typescript --out-dir tm-server-manager-api-ts/server-manager --module-path tm-server-manager