# SQL client

A SQL database client to run a query

## Usage

```sh
pen build
./app query postgresql://localhost:5432/database 'SELECT * FROM foo;'
./app execute postgresql://localhost:5432/database "INSERT INTO foo VALUES (1, 'bar');"
```
