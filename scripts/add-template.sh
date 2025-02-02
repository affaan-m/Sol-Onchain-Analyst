#!/bin/bash

# Check if the name is provided
if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <plugin_name> <rig_name> <example_name>"
    exit 1
fi

PLUGIN_NAME=$1
RIG_NAME=$2
EXAMPLE_NAME=$3

# Create cainam-plugins
cargo new --lib cainam-plugins/$PLUGIN_NAME
# Rename package name in Cargo.toml
sed -i "s/name = \"$PLUGIN_NAME\"/name = \"cainam-plugin-$PLUGIN_NAME\"/" cainam-plugins/$PLUGIN_NAME/Cargo.toml

# Create examples
cargo new examples/$EXAMPLE_NAME