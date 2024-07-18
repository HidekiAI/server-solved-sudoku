#!/bin/sh

# verify-kafka.sh
set -e

host="$1"
port="$2"
shift 2
cmd="$@"

# Attempt to connect to Kafka
until nc -z "$host" "$port"; do
  echo "Waiting for Kafka at $host:$port..."
  sleep 5
done

echo "Kafka is reachable at $host:$port"

# Optionally, produce a test message to verify the Kafka broker
kafka-topics.sh --create --topic test-topic --bootstrap-server "$host:$port" --partitions 1 --replication-factor 1
kafka-console-producer.sh --broker-list "$host:$port" --topic test-topic <<< "test message"

exec $cmd
