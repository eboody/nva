FROM postgres:17-alpine

WORKDIR /workspace

COPY migrations ./migrations
COPY fixtures/seed ./fixtures/seed

CMD ["/bin/sh", "-ec", "for migration in /workspace/migrations/*.sql; do\n  echo \"[migrate-seed] applying $${migration}\"\n  psql -v ON_ERROR_STOP=1 -f \"$${migration}\"\ndone\necho \"[migrate-seed] applying synthetic local demo seed\"\npsql -v ON_ERROR_STOP=1 -f /workspace/fixtures/seed/local-demo.sql\npsql -v ON_ERROR_STOP=1 -f /workspace/fixtures/seed/local-demo-data-quality.sql\necho \"[migrate-seed] local schema and seed are ready\"\n"]
