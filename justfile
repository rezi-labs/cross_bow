import 'docker.just'
import? 'private.just'

image_name := "ghcr.io/rezi-labs/cross_bow"

export LOCAL := "true"
export POSTGRES_PASSWORD := "postgres"
export POSTGRES_USER := "postgres"
export POSTGRES_DB := "taste"

docker:
    docker compose up

run: start-db
    cargo run

watch:
    cargo watch -x run

verify: lint test

test:
    cargo test

lint:
    cargo fmt --all -- --check
    cargo clippy

fmt:
    cargo fmt
    cargo fix --allow-dirty --allow-staged

generate-session-secret:
    openssl rand -base64 64


# Start PostgreSQL database with persistent volume
start-db: stop-db
    docker run -d \
        --name taste-postgres \
        -e POSTGRES_PASSWORD={{POSTGRES_PASSWORD}} \
        -e POSTGRES_USER={{POSTGRES_USER}} \
        -e POSTGRES_DB={{POSTGRES_DB}} \
        -p 5500:5432 \
        -v taste-postgres-data:/var/lib/postgresql/data \
        postgres:18-alpine

# Stop PostgreSQL database
stop-db:
    docker stop taste-postgres || true
    docker rm taste-postgres || true

# Wipe PostgreSQL database volume (removes all data)
wipe-db-volume: stop-db
    docker volume rm taste-postgres-data || true
