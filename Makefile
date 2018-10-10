all:
	docker-compose build

server:
	docker-compose build simple-secrets

env: all
	docker-compose down
	docker-compose up -d
	./boostrap-etcd.sh
	./register-spire-policy.sh
	./register-spire-agents.sh

clean:
	docker-compose down -v
	docker-compose rm

.PHONY: all server env
