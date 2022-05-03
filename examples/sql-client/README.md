# SQL client

A SQL database client to run a query

## Usage

```sh
pen build
./app query postgresql://localhost:5432/database 'SELECT * from foo;'
./app execute postgresql://localhost:5432/database "insert into foo values (1, 'bar');"
```
