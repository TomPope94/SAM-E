#!/usr/bin/env bash

sudo SAM_TEMPLATE="$(find "template.yaml" | xargs yq eval-all '. as $item ireduce ({}; . * $item )')" AWS_PROFILE="staging-mfa" docker compose up --build
