# Redis cursor

This example shows you how to persits the cursor information on a Redis cluster.

The `daemon.toml` includes the `[cursor]` section that has `type` set to `Redis`, `key` is the key on the redis cluster where to dump the information, and `url` the connection string to connect with Redis.

To run the example:

* Set up [Demeter CLI](https://docs.demeter.run/cli).
* On a different terminal but same path run `dmtr ports tunnel`.
  Chose the `node` option, followed by the `preprod` network and the `stable` version. Finally, mount the socket on `./socket`.
* `docker compose up -d` to spin up the Redis instance.
* ```sh
  cargo run --bin oura --features redis daemon --config daemon.toml
  ```

In order to see cursor information on the Redis you can do the following.

```sh
$ docker exec -it redis redis-cli
127.0.0.1:6379> GET key
```