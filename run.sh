#!/usr/bin/env bash

CONFIG="$(find "sam-e-config.yaml" | xargs yq eval-all '. as $item ireduce ({}; . * $item )')" docker compose --compatibility up --build
