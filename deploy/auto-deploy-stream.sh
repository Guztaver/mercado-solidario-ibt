#!/usr/bin/env bash
set -Eeuo pipefail

PROJECT_DIR="/srv/docker/mercado-solidario-ibt-source"
RUNTIME_DIR="/srv/docker/mercado-solidario-ibt"
DEPLOYED_FILE="$RUNTIME_DIR/.deployed-commit"
CONTAINER="mercado-solidario-ibt"
IMAGE_REPO="mercado-solidario-ibt"
LOG_PREFIX="mercado-solidario-ibt deploy:"

mkdir -p "$RUNTIME_DIR"
exec 9>"/var/lock/mercado-solidario-ibt-deploy.lock"
flock -n 9 || exit 0

cd "$PROJECT_DIR"
git fetch --quiet origin main
target="$(git rev-parse origin/main)"
deployed="$(cat "$DEPLOYED_FILE" 2>/dev/null || true)"

if [[ "$target" == "$deployed" ]]; then
    exit 0
fi

if [[ -n "$deployed" ]]; then
    previous_image="$IMAGE_REPO:$deployed"
    if ! docker image inspect "$previous_image" >/dev/null 2>&1; then
        running_image="$(docker inspect --format '{{.Image}}' "$CONTAINER")"
        docker image tag "$running_image" "$previous_image"
    fi
else
    echo "$LOG_PREFIX no previous deployment marker; refusing untracked initial deployment" >&2
    exit 1
fi

target_image="$IMAGE_REPO:$target"
git reset --hard "$target" >/dev/null
install -m 0644 compose.stream.yaml "$RUNTIME_DIR/compose.stream.yaml.new"
mv -f "$RUNTIME_DIR/compose.stream.yaml.new" "$RUNTIME_DIR/compose.stream.yaml"
docker build --build-arg RUN_TESTS=false --tag "$target_image" "$PROJECT_DIR"

backup_tmp="/data/backups/.before-${target}.tmp.db"
backup_final="/data/backups/before-${deployed}.db"
docker exec "$CONTAINER" mkdir -p /data/backups
docker exec "$CONTAINER" rm -f "$backup_tmp"
docker exec "$CONTAINER" mercado-solidario backup "$backup_tmp"
docker exec "$CONTAINER" mv -f "$backup_tmp" "$backup_final"

APP_IMAGE="$target_image" docker compose --project-directory "$RUNTIME_DIR" --file "$RUNTIME_DIR/compose.stream.yaml" up -d --no-deps app

healthy=false
for _ in $(seq 1 36); do
    state="$(docker inspect --format '{{.State.Status}}/{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' "$CONTAINER" 2>/dev/null || true)"
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
    APP_IMAGE="$previous_image" docker compose --project-directory "$RUNTIME_DIR" --file "$RUNTIME_DIR/compose.stream.yaml" up -d --no-deps app
    for _ in $(seq 1 24); do
        state="$(docker inspect --format '{{.State.Status}}/{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' "$CONTAINER" 2>/dev/null || true)"
        [[ "$state" == "running/healthy" ]] && break
        sleep 5
    done
    exit 1
fi

printf '%s\n' "$target" > "$DEPLOYED_FILE.new"
mv -f "$DEPLOYED_FILE.new" "$DEPLOYED_FILE"
while IFS= read -r image; do
    if [[ "$image" != "$target_image" && "$image" != "$previous_image" ]]; then
        docker image rm "$image" >/dev/null 2>&1 || true
    fi
done < <(docker image ls --format '{{.Repository}}:{{.Tag}}' | grep -E "^${IMAGE_REPO}:[0-9a-f]{40}$" || true)
echo "$LOG_PREFIX deployed $target_image"
