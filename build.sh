#!/bin/bash
# Convention:
# - When I mean "Linux", it's both actual Linux distro (I am biased on Debian) or WSL2 (again, Debian biased)
# - VCPKG_ROOT for Linux will be on `/opt/vcpkg` (where binary is at `/opt/vcpkg/vcpkg`)
# - VCPKG_ROOT for Windows will be on `C:\msys64\opt\vcpkg\` (where binary is at `C:\msys64\opt\vcpkg\vcpkg.exe` )
# - With that said, it is assume that for Windows, you're working with MSYS2 (mingw64) and not Cygwin
# - Assume rustup is installed whether on Windows or Linux
# - Docker && Docker-compose is installed (for both Linux or Windows)
# - Do NOT install/enable kubernetes on Windows unless you hate yourself or you get paid gobs of $$$ by the hour
#   If you do have it installed, disabled it and delete all containers and images associated to it such as kube and pods
#   (for REAL Linux, you can use/keep kubernetes, but for WSL2, just don't!)
# - You've already edited .env.local with your own values from Google Cloud Platform
set -o nounset   # exit when your script tries to use undeclared variables   

function make_mingw() {
    echo "mingw-w64-x86_64-${1}"
}

_CARGO=$(which cargo 2>&1 | grep -v "not found")
if [ "${_CARGO}" == "" ] ; then
    echo "cargo not found, please install Rust"
    echo "#$ curl https://sh.rustup.rs -sSf | sh"
    echo " or for Windows, go to: https://win.rustup.rs/"
    exit -1
fi

# first, install base dependencies either for MinGW or Linux
if [ "$(uname -o)" == "Msys" ] ; then
    echo "# Building on MSYS/MinGW: $(uname -a)"
    pacman --sync --noconfirm           \
        zip unzip                       \
        $(make_mingw "toolchain")       \
        $(make_mingw "gcc")             \
        $(make_mingw "gcc-libs")        \
        $(make_mingw "pkg-config")      \
        $(make_mingw "pkgconf")         \
        $(make_mingw "make") $(make_mingw "cmake") \
        $(make_mingw "zstd") libzstd libzstd-devel zstd \
        libsqlite-devel

    # Install vcpkg
    export VCPKG_ROOT=/opt/vcpkg
    if ! [ -e "${VCPKG_ROOT}/vcpkg" ] ; then
        sudo mkdir -p "${VCPKG_ROOT}"
        sudo chmod +rwx "${VCPKG_ROOT}"
        sudo chown $(whoami) "${VCPKG_ROOT}"
        git clone https://github.com/microsoft/vcpkg.git "${VCPKG_ROOT}" && "${VCPKG_ROOT}"/bootstrap-vcpkg.sh
    fi
    export PATH="${VCPKG_ROOT}:${PATH}"
    export PKG_CONFIG_PATH=${VCPKG_ROOT}/packages
    cargo install cargo-vcpkg
    # NOTE: On MinGW64, you `vcpkg install librdkafka:x64-mingw-dynamic`
    #       On Linux, you `vcpkg install librdkafka:x64-linux`
    #       On Windows, you `vcpkg install librdkafka:x64-windows` BUT without VStudios installed, it WILL FAIL, so use mingw instead!
    vcpkg install librdkafka:x64-mingw-dynamic
    pacman --sync --noconfirm           \
        $(make_mingw "librdkafka")
    
elif [ "$(uname -o)" == "GNU/Linux" ] ; then
    # Linux
    echo "# Building on Linux: $(uname -a)"
    sudo apt-get install -y --install-recommends \
        zip unzip curl \
        gcc g++ cmake make \
        build-essential \
        pkg-config  \
        libssl-dev libsasl2-dev libzstd-dev
    sudo apt-get install -y --install-recommends \
        protobuf-compiler \
        libprotobuf-dev \
        grpc-proto 

    # Note that librdkafka-dev is ONLY the header files; we need the actual library (binary)
    # as well, so we'll do "librdkafka*" to get all the libraries (using willcard instead of)
    # versioned package name makes it easier to maintain (though it's a careless action
    # because next version up may not be compatible)...
    sudo apt-get install -y --install-recommends \
        libssl-dev      \
        librdkafka*     \
        libsqlite3-dev  \
        libpq-dev       
    # Install vcpkg
    export VCPKG_ROOT=/opt/vcpkg
    if ! [ -e "${VCPKG_ROOT}/vcpkg" ] ; then
        sudo mkdir -p "${VCPKG_ROOT}"
        sudo chmod +rwx "${VCPKG_ROOT}"
        sudo chown $(whoami) "${VCPKG_ROOT}"
        git clone https://github.com/microsoft/vcpkg.git "${VCPKG_ROOT}" && "${VCPKG_ROOT}"/bootstrap-vcpkg.sh
    fi
    export PATH="${VCPKG_ROOT}:${PATH}"
    export PKG_CONFIG_PATH=${VCPKG_ROOT}/packages
    cargo install cargo-vcpkg
    # NOTE: On MinGW64, you `vcpkg install librdkafka:x64-mingw-dynamic`
    #       On Linux, you `vcpkg install librdkafka:x64-linux`
    #       On Windows, you `vcpkg install librdkafka:x64-windows` BUT without VStudios installed, it WILL FAIL, so use mingw instead!
    _BIN_FOUND=$(which vcpkg 2>&1 | grep -vi "error" )
    if [ "${_BIN_FOUND}" == "" ] ; then
        echo "Unable to locate vcpkg!"
        exit -1
    fi
    vcpkg install librdkafka:x64-linux
else
    echo "# You're using an O/S that understands 'uname' $(uname -o) command, but I don't know what it is... $(uname -a)"
    # Windows or some other OS I care not about...
    echo "OS not supported"
    exit -1
fi

# NOTE: Quite a few crates relies on vcpkg, hence MAKE SURE
#       - VCPKG_ROOT is set to /opt/vcpkg
#       - make sure PATH has ${VCPKG_ROOT} in the search (i.e. `export PATH=${VCPKG_ROOT}:$PATH`)
# Prior to calling `cargo build`, let's make sure that vcpkg is accessible:
_BIN_FOUND=$(which vcpkg 2>&1 | grep -vi "error" )
if [ "${_BIN_FOUND}" == "" ] ; then
    echo "Unable to locate vcpkg!"
    exit -1
fi
if ! [ -e "${VCPKG_ROOT}/vcpkg" ] ;  then
    echo "Directory ${VCPKG_ROOT} is inaccessible!"
    exit -1
fi

if [ "$(uname -o)" == "GNU/Linux" ] ; then
    echo "######################################## Build Linux"
    
    #cargo build --release --target-dir target-gnu-linux
    #cargo build --release --target-dir target
    cargo build --release
elif [ "$(uname -o)" == "Msys" ] ; then
    echo "######################################## Build MinGW"
    echo "WARNING: WIll not be able to build Docker images on Windows, you'll need to build INSIDE Docker container that is Linux-based"
    cargo build --release --target-dir target-msys
else
    echo "OS not supported"
    exit -1
fi

pushd . 2>&1 > /dev/null
cd micro-services
./make-service.sh
popd 2>&1 > /dev/null