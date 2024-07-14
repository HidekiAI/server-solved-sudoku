#!/bin/bash +x
# Original intentions were to call "cargo build" for each Dockerfile micro-services
# but it turns out that if your Cargo.toml uses "path" for dependencies, in
# which the directory is above (i.e. "../libs") the current directory, then
# Docker will not allow it.
# I cannot juggle directories (i.e. copy "../libs" to "./build/libs") because
# it'll mess with building (via "cargo build") local versus Dockerfile.
# So in the end, my solution is to just "cargo build --release" OUTSIDE of Docker
# (basically, at the root of the project) and then copy the "target/release" binaries
# into the Dockerfile's build context.
set -o nounset   # exit when your script tries to use undeclared variables   

# Assume this script is on ${_ROOT_DIR}/micro-services/make-service.sh
_PWD=$(pwd)
pushd . 2>&1 > /dev/null
cd ..
_ROOT_DIR=$(pwd)
_MICRO_SERVICE_DIR=${_ROOT_DIR}/micro-services

if [ "$(uname -o)" == "GNU/Linux" ] ; then
    echo "######################################## Build Linux services (including WSL2)"
elif [ "$(uname -o)" == "Msys" ] ; then
    _WIN_DOCKER=$( which docker.exe 2>&1 2>/tmp/out.txt && grep "\/Docker" /tmp/out.txt )
    if [ "${_WIN_DOCKER}" == "" ]; then
        echo "# Make sure Dockers for Windows is in the search paths"
        exit -1
    fi
    echo "Detected MSys/MinGW64 environment...  Hand-crafting .env files to make it work on Windows Docker-Compose..."
    # first, remove all comments, in which I've purposely added comments on lines that Windows Docker-Compose dislikes
    # Note that this is NOT an issue on MinGW-BASH (which can do nested source'ing as well as accept BASH commands 
    # INSIDE source'ing file).  It's very deceptive because it allows BASH command `export` in .env, but won't allow
    # any other BASH commands.
    # TODO: Perhaps I should have 2 files, one is ".env.sh" and another is ".env" in which ".env" is in format of
    # TODO: 'Key=Value' that Docker-Compose understands, and ".env.sh" is the version we would "source" with, hence
    # TODO: ".env.sh" will `source .env` in from BASH, you'd do `source .env.sh`; The KVP ".env" file is basically ".env.msys" below:
    cd ${_ROOT_DIR}
    grep -v "#" .env > .env.msys
    # append overriding vars to .env.sys
    cat .env.local >> .env.msys
else
    echo "OS not supported"
    exit -1
fi
pwd

# Next, we'll need to source the '.env' so that all the auto-generated scripts
# and logics (including Dockerfile) will have access to the environment variables.
cd ${_MICRO_SERVICE_DIR}
if ! [ -e .env.local ] ; then
    echo "# Please create local file '.env.local' and at least set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET"
    echo "# the file '.env.local' must reside on the same directory as '.env' file"
    exit -1
else
    # if .env.local exists, assume all overrides are set there, so make sure .env is in its prestine conditions
    git fetch --all 2>&1 2> /dev/null
    _DIFF=$(git diff .env)
    if [ "${_DIFF}" != "" ] ; then
        echo "############################################"
        git diff .env
        echo "############################################"
        echo "# .env is modified, reset to original using:"
        echo '$ git reset .env && git checkout .env'
        exit -1
    fi
fi

# NOTE: Must have env set, i.e. `$ source .env && docker-compose --verbose --ansi=auto build` because
# docker-compose.yml uses ${VAR} syntax, and it will not be expanded if not set in the environment
if [ -e .env.msys ]; then
    source .env.msys
else
    source .env
fi

if [ "${GOOGLE_CLIENT_ID}" == "your_google_client_id" ] ; then
    echo "# Please update GOOGLE_CLIENT_ID in your .env.local"
    exit -1
fi

if [ "${GOOGLE_CLIENT_SECRET}" == "your_google_client_secret" ] ; then
    echo "# Please update GOOGLE_CLIENT_SECRET in your .env.local" 
    exit -1
fi

# Let's next make sure both docker-compose and docker are installed and running (as a service)
_DOCKER=$(which docker 2>&1 | grep -v "failed")
if ! [ -e "${_DOCKER}" ]; then
    echo "# Install docker prior to building"
    exit -1
fi
if ! [ -e "$(which docker-compose)" ]; then
    echo "# Install docker (docker-compose) prior to building"
    exit -1
fi
sudo service docker status | tee ./.docker-status.txt
if grep -q "failed" ./.docker-status.txt ; then
    echo "# Start docker service prior to building"
    echo '$ sudo service docker start'
    exit -1
fi

if [ "${VCPKG_ROOT}" == "" ] ; then
    echo "# Please set VCPKG_ROOT to the directory where vcpkg is installed"
    exit -1
fi
if [ "${PKG_CONFIG_PATH}" == "" ] ; then
    echo "# Please set PKG_CONFIG_PATH to the directory where librdkafka.pc is installed"
    exit -1
fi

# Finally, build the entire project (including the micro-services)
# libs and micro-services binaries will end up in target/release
echo "# Building micro-services on $(pwd)"
find ${VCPKG_ROOT} -name "*kafka*pc"
find ${PKG_CONFIG_PATH} -name "*kafka*pc"
if [ "$(uname -o)" == "GNU/Linux" ] ; then
    echo "######################################## Build Linux services"
    #cargo build --release --target-dir target-gnu-linux
    cargo build --release
elif [ "$(uname -o)" == "Msys" ] ; then
    #cargo build --release --target-dir target-msys
    echo "Cannot build Docker images on Windows, you'll need to build INSIDE Docker container that is Linux-based"
else
    echo "OS not supported"
    exit -1
fi

# Strategies of copying file:
# - One of the issue with using ".env.local" to hold sensitive files (for Dockerfile)
#   and Config::from_env() is that we have to copy the ".env.local" to each
#   micro-service's directory. This is because Docker will not allow access to
#   dirs above it's build dir.
# - With that said, we'll use soft-link (ln -s) to link the ".env.local" and
#   ".env" to each micro-service's directory, but we'll keep track of WHERE it was
#   linked from (i.e. the root micro-services dir) so that we can remove it later.
#   Note that Windows MinGW ln actually makes a copy instead of soft-link but
#   that's fine because it's not a dir we're linking, so link or file, it's just
#   a single file (and should be treated the same).
# Walk through each micro-service and look for 'Dockerfile', in which if found,
# we'll create a directory "build" and copy the necessary files for building.
# and copy './target/release' binaries to the build dir as well as
# the '.env' and '.env.local' files. (ALL micro-services will assume that
# the '.env' and '.env.local' are in found in ./build/.env ./build/.env.local
# during runtime).  See `Config::from_env("./build/.env")` as well as internal
# implementation of `Config::from_env()` for more info (i.e. issue on path).
_TARGET=${_ROOT_DIR}/target/release
_DIRS=$( find . -type f -name "Dockerfile" -exec dirname {} \; )
for _D in ${_DIRS} ; do
    echo "# Setting up micro-service in ${_D}"
    pushd . 2>&1 > /dev/null
    cd ${_D}
    mkdir -p build
    cd build
    # Because Docker will not allow access to dirs above it's build dir,
    # we will need to copy the .env.local to the build dir.
    # See oauth_relay_service/Dockerfile for more info.
    ln -sv ${_MICRO_SERVICE_DIR}/.env
    ln -sv ${_MICRO_SERVICE_DIR}/.env.local
    # NOTE: No need to recursively copy, just copy the binaries at the root of the target/release
    cp --update ${_ROOT_DIR}/target/release/* .
    popd 2>&1 > /dev/null
done

# NOTE: The Dockerfile for oauth_relay_service will perform multi-stage build so that libscsudoku will be built when oauth_relay_service is built.
docker-compose --verbose --ansi=auto build --progress=plain

# CLEAN UP
# just in case somebody decided to add it to local commit, remove it!
git rm .env.local 2>&1 > /dev/null
git rm --cached .env.local 2>&1 > /dev/null

cd ${_MICRO_SERVICE_DIR}
echo "# Cleaning up duplicated .env* files"
find . -name ".env*"
_DIRS=$( find . -type f -name "Dockerfile" -exec dirname {} \; )
for _D in ${_DIRS} ; do
    rm ${_D}/build/.env
    rm ${_D}/build/.env.local
done
cd ${_ROOT_DIR}
if [ -e .env.msys ]; then
    rm .env.msys
fi
echo "# After cleaning up duplicated .env* files:"
find . -name ".env*"

popd 2>&1 > /dev/null