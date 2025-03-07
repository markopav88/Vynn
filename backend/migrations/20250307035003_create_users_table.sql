-- Add migration script here
-- For users
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE
);
-- interact with your PostgreSQL instance
-- 1.Connect via psql inside the container: Run: docker exec -it contained-postgres psql -U postgres -d my_app_db
-- 2. To list the tables: Run: \dt(Nothing inside rn, nothing to return)
-- 3 \q to exit

