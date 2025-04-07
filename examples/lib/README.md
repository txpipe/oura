# Example using Oura Lib

Oura lib allows custom stages. In this example there are two custom stages, one is to filter only transactions used to delegate a drep vote and the second is a sink that persists values in a sqlite.

For this example, it's required to have the sqlx cli to prepare a local sqlite db. Oura is using gasket version 0.7 for this example.

Before executing the code, expose a env to define the path of the db. Create a file `.env` with the values below.

```.env
DATABASE_URL="sqlite:local.db"
RUST_LOG="info"
```

Execute `cargo run` to execute. The source will connect to cardano foundation node using N2N source.