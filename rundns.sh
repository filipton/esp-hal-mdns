#!/bin/bash

# if venv doesnt exists create it
if [ ! -d ./.venv ]; then
    echo "Creating virtual environment"
    python3 -m venv ./.venv
fi

source ./.venv/bin/activate

pip3 install dnserver
dnserver --port 5053 ./testzones.toml
