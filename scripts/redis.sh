# Starter commands for redis script(s)
docker run --name some-redis -d redis redis-server --save 60 1 --loglevel warning
docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' some-redis
redis-cli -h 172.17.0.3
