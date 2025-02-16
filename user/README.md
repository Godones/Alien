# Run app


## redis
```
server: redis-server ./bin/redis.conf &
bench : redis-benchmark -n 1000 -c 1
client: redis-cli
```