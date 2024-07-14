#!/bin/bash

if ! [ -e .env.local ] ; then
    echo "# Please create local file '.env.local' and at least set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET"
    echo "# the file '.env.local' must reside on the same directory as '.env' file"
    exit -1
else
    # if .env.local exists, assume all overrides are set there, so make sure .env is in its prestine conditions
    git fetch --all 2>&1 2> /dev/null
    git reset .env
    git checkout .env
fi
source .env

if [ "$GOOGLE_CLIENT_ID" == "your_google_client_id" ] ; then
    echo "# Please update GOOGLE_CLIENT_ID in your .env.local"
    exit -1
fi

if [ "GOOGLE_CLIENT_SECRET" == "your_google_client_secret" ] ; then
    echo "# Please update GOOGLE_CLIENT_SECRET in your .env.local" 
    exit -1
fi

if ! [ -e $(which docker-compose) ]; then
    echo "# Install docker (docker-compose) prior to building"
    exit -1
fi
docker-compose build

# just in case somebody decided to add it to local commit, remove it!
git rm .env.local 2>&1 > /dev/null
git rm --cached .env.local 2>&1 > /dev/null
