# Example using Oura Lib

Oura lib allows custom stages. In this example there are two custom stages, one is to filter only transactions used to delegate a drep vote and the second is a sink that persists values in a sqlite.

For this example, it's required to have the `sqlx cli` to prepare a local sqlite db. Oura is using gasket version 0.7 for this example.

To install the sqlx cli, use the cargo install

```sh
 cargo install sqlx-cli
```

Follow the command below to prepare the environment to run this example

```sh
# Set db path env
export DATABASE_URL="sqlite:local.db"

# Create database
cargo sqlx db create

# Start migrations
cargo sqlx migrate run --source ./src/migrations
```

Execute `cargo run` to execute. The source will connect to cardano foundation node using N2N source.