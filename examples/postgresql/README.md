# PostgreSQL Sink Example

## Steps

1. initialize docker compose

```
docker-compose up
```

2. create required tables

```
psql -h localhost -p 5432 -U postgres -f init.sql
```