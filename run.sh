#!/usr/bin/env bash

sudo SAM_TEMPLATE="$(find "template.yaml" | xargs yq eval-all '. as $item ireduce ({}; . * $item )')" docker compose --compatibility up --build
