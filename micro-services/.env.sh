#!/bin/bash
# NOTE: This is NOT for Docker-Compose and/or any of the Python-based dotenv stuff, I have
# a separate script 'make_env.sh' which will convert this '.env.sh' and '.env.local' to
# '.env' file.  But if you're just SOURCE'ing, you can just do 'source .env.sh' (or '. .env.sh')
# of THIS file and should work just fine.
set -o nounset

# NOTE: DO _NOT_ commit this file to the repository IF it contains sensitive information!
# This file should NEVER be edited, it's here to express/define environment variables
# needed by Docker-compose, etc, and placed with default value.
# All sensitive information should be stored in .env.local
# which is part of .gitignore and should NEVER be committed to the repository!

# Following are the environment variables MUST be defined (exported) in .env.local
# in which you can obtain from Google Developer Console (https://console.developers.google.com)
# Sample .env.local:
#   export GOOGLE_CLIENT_ID="abcdef"
#   export GOOGLE_CLIENT_SECRET="123456"
GOOGLE_CLIENT_ID=your_google_client_id

GOOGLE_CLIENT_SECRET=your_google_client_secret

# Although Google accepts http, ideally we want https...  This URL
# is what Google instructs client to redirect itself to after
# successful authentication.  This MUST MATCH exactly (all the way
# up to trailing "/" (if exists)) or else you will get a redirect
# failure error from Google!  I'm not sure why it works, but
# using http://localhost seems to work on non-Docker (NAT actually)
# dev hosts...  On Docker-compose based, I think it still works as well
# due to port-forwarding, though I've never been successful with that...
REST_PORT=8080
GOOGLE_REDIRECT_URI=http://localhost:${REST_PORT}/auth_callback

# Database connection information, for PostgreSQL we need host:port but for sqlite, all we need is the path to the file
# As for username/passwd for SQL services, it should be at the host access level (i.e. in MySQL, it's via I.P. address)
# I do agree that CIDN-IP-based access is not the most secure, but I do not wish to over-complicate this project (personally
# I think stuffing password in a static file here is worse!)
# NOTE: For SQLite, the directory MUST exist (but if file does not exist, it will be created), and at the same
# time, if it is within the Docker container, it will be created as a volume (so it will persist)
DB_CONNECTION=sqlite
DB_HOST=localhost
DB_PORT=5432
DB_STORAGE_PATH=./data/db.sqlite3

# Message broker connection information, for RabbitMQ we need host:port but for Redis, all we need is the path to the file
# Kafka: 9092
# RabbitMQ (AMQP): 5672
# Redis: 6379
# Make sure the hostname (BROKER_HOST) matches whats on Docker-Compose hostname
MQ_CONNECTION=kafka
BROKER_HOST=kafka_auth_messenger
BROKER_PORT=9092

# Now load local environment variables, if it exists
# NOTE: Unfortunately, Docker-compose build seems to use this non-POSIX
# env approach (I think it's written in python?) that will NOT allow
# nested source'ing, hence when running under Windows (MSys) that is using
# Docker at path 'C:\Program Files\Docker\Docker\resources\bin', you'll
# need to comment this and do:
#   $ source .env && source .env.local
source .env.local       # nested source'ing...

# In case REST_PORT was overridden in .env.local, we'll need to update this as well:
GOOGLE_REDIRECT_URI=http://localhost:${REST_PORT}/auth_callback

export REST_PORT=$REST_PORT

export GOOGLE_CLIENT_ID=$GOOGLE_CLIENT_ID
export GOOGLE_CLIENT_SECRET=$GOOGLE_CLIENT_SECRET
export GOOGLE_REDIRECT_URI=$GOOGLE_REDIRECT_URI

export DB_CONNECTION=$DB_CONNECTION
export DB_HOST=$DB_HOST
export DB_PORT=$DB_PORT
export DB_STORAGE_PATH=$DB_STORAGE_PATH

export MQ_CONNECTION=$MQ_CONNECTION
export BROKER_HOST=$BROKER_HOST
export BROKER_PORT=$BROKER_PORT
