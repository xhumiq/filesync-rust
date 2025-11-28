#!/bin/bash

# Default profile
PROFILE=${1:-.env.local}

echo "Using profile: $PROFILE"

# Export the environment variable
export ENV_PROFILE="$PROFILE"

# Also try the CARGO_ prefix
export CARGO_ENV_PROFILE="$PROFILE"

# Run trunk serve with the environment variable
exec ~/.cargo/bin/trunk serve --port 3030 --dist ./dist/debug