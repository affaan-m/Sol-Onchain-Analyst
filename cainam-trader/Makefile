.PHONY: migrate run-dev

migrate:
	cargo run --bin run_migrations

run-dev:
	cargo run

.PHONY: docker-postgres
docker-postgres:
	docker run -e POSTGRES_USER=mgunnin \
           -e POSTGRES_PASSWORD=password \
           -e POSTGRES_DB=cainam_db \
           --name cainam_postgres \
           -p 5432:5432 \
           -d ankane/pgvector 