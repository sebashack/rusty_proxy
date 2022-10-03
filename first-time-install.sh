#!/bin/bash

set -xeuf -o pipefail

sudo apt install curl build-essential
curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh -s -- -y

rustup self update
rustup default stable
