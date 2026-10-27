#!/usr/bin/env bash
set -Eeuo pipefail

PROJECT_DIR="/home/gustavo/mercado-solidario-source"
RUNTIME_DIR="/home/gustavo/mercado-solidario"
DEPLOYED_FILE="$RUNTIME_DIR/.deployed-commit"
LOG_PREFIX="mercado-solidario deploy:"

cd "$PROJECT_DIR"
git fetch --quiet origin main
target="$(git rev-parse origin/main)"
deployed="$(cat "$DEPLOYED_FILE" 2>/dev/null || true)"

if [[ "$target" == "$deployed" ]]; then
    exit 0
fi

if [[ -n "$deployed" ]]; then
    previous_image="mercado-solidario:$deployed"
    if ! docker image inspect "$previous_image" >/dev/null 2>&1; then
        running_image="$(docker inspect --format '{{.Image}}' mercado-solidario)"
        docker image tag "$running_image" "$previous_image"
    fi
else
    echo "$LOG_PREFIX no previous deployment marker; refusing untracked initial deployment" >&2
    exit 1
fi

target_image="mercado-solidario:$target"
git reset --hard "$target" >/dev/null
APP_IMAGE="$target_image" docker compose --project-name mercado-solidario build --build-arg RUN_TESTS=false app

backup_tmp="/data/backups/autodeploy-latest.new.db"
backup_final="/data/backups/autodeploy-latest.db"
docker exec mercado-solidario mercado-solidario backup "$backup_tmp"
docker exec mercado-solidario mv -f "$backup_tmp" "$backup_final"

APP_IMAGE="$target_image" docker compose --project-name mercado-solidario up -d --no-deps app

healthy=false
for _ in $(seq 1 36); do
    state="$(docker inspect --format '{{.State.Status}}/{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' mercado-solidario 2>/dev/null || true)"
    if [[ "$state" == "running/healthy" ]]; then
        healthy=true
        break
    fi
    if [[ "$state" == exited/* || "$state" == dead/* ]]; then
        break
    fi
    sleep 5
done

if [[ "$healthy" != true ]]; then
    echo "$LOG_PREFIX $target failed health check ($state); rolling back to $deployed" >&2
    APP_IMAGE="$previous_image" docker compose --project-name mercado-solidario up -d --no-deps app
    for _ in $(seq 1 24); do
        state="$(docker inspect --format '{{.State.Status}}/{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' mercado-solidario 2>/dev/null || true)"
        [[ "$state" == "running/healthy" ]] && break
        sleep 5
    done
    exit 1
fi

printf '%s\n' "$target" > "$DEPLOYED_FILE.new"
mv -f "$DEPLOYED_FILE.new" "$DEPLOYED_FILE"
echo "$LOG_PREFIX deployed $target_image"
