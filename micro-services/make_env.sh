#!/bin/bash
# Arg1: Optional - Source file (default: .env.sh)
# Arg2: Optional - Local file (default: .env.local)
# Generates a 'build/.env' file from '.env.sh' and '.env.local' files 
# The python-based dotenv stuff (Windows and Linux) only supports KVP (Key=Value) format
# and it is NOT a POSIX shell script, so we'll need to convert the .env.sh to .env
# which is in KVP format that Docker-Compose understands.
_SOURCE=".env.sh"
_LOCAL=".env.local"
_DEST="./build/.env"

if [ "${1}" != "" ]; then
    _SOURCE=${1}
    shift
fi
if [ "${1}" != "" ]; then
    _LOCAL=${1}
    shift
fi
set -o nounset   # exit when your script tries to use undeclared variables

if ! [ -e "${_SOURCE}" ] ; then
    echo "ERROR: ${_SOURCE} not found!"
    exit -1
fi
if ! [ -e "${_LOCAL}" ] ; then
    echo "ERROR: ${_LOCAL} not found!"
    echo "# Please create local file '.env.local' and at least set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET"
    echo "# the file '.env.local' must reside on the same directory as '.env' file"
    exit -1
else
    pushd . 2>&1 > /dev/null
    cd $(dirname ${_SOURCE})
    # if .env.local exists, assume all overrides are set there, so make sure .env is in its prestine conditions
    git fetch --all 2>&1 2> /dev/null
    _DIFF=$(git diff $(basename ${_SOURCE}))
    if [ "${_DIFF}" != "" ] ; then
        echo "############################################"
        git diff $(basename ${_SOURCE})
        echo "############################################"
        echo "# $(basename ${_SOURCE}) is modified, reset to original using:"
        echo "$ git reset $(basename ${_SOURCE}) && git checkout $(basename ${_SOURCE})"
        exit -1
    fi
    popd 2>&1 > /dev/null

    _FOUND_DEFAULT=$( grep -v "#" ${_LOCAL} | grep "your_google_client_id" )
    if [ "${_FOUND_DEFAULT}" != "" ] ; then
        echo "ERROR: GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET must be set in '.env.local' file"
        exit -1
    fi
fi

if ! [ -e "$(dirname ${_DEST})" ] ; then
    mkdir -p "$(dirname ${_DEST})"
fi

# Note that we assume all lines in '.env.sh' that has "#" comment such as:
#   'set -o nounset   # exit when your script tries to use undeclared variables'
# will be removed; not that in '.env.sh' file, I've INTENTIONALLY added comments
# to lines that python dotenv disagrees with.
grep -v "#" ${_SOURCE} > ${_DEST}
# now post-end the '.env' file with the '.env.local' file so that all the local
# assignes will be overridden:
grep -v "#" ${_LOCAL} >> ${_DEST}

echo "# Created '${_DEST}' file @ $(pwd) based on '${_SOURCE}' and '${_LOCAL}'"