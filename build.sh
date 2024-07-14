#!/bin/bash
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
    pacman --sync --noconfirm           \
        zip unzip                       \
        $(make_mingw "gcc")             \
        $(make_mingw "gcc-libs")        \
        $(make_mingw "pkg-config")      \
        $(make_mingw "pkgconf")         \
        $(make_mingw "make") $(make_mingw "cmake") \
        libsqlite-devel

    # Install vcpkg
    if ! [ -e /opt/vcpkg ] ; then
        mkdir -p /opt/vcpkg
        git clone https://github.com/microsoft/vcpkg.git /opt/vcpkg && /opt/vcpkg/bootstrap-vcpkg.sh
    fi
    PATH="/opt/vcpkg:${PATH}"
    cargo install cargo-vcpkg
    # NOTE: On MinGW64, you `vcpkg install librdkafka:x64-mingw-dynamic`
    #       On Linux, you `vcpkg install librdkafka:x64-linux`
    #       On Windows, you `vcpkg install librdkafka:x64-windows` BUT without VStudios installed, it WILL FAIL, so use mingw instead!
    vcpkg install librdkafka:x64-mingw-dynamic
    pacman --sync --noconfirm           \
        $(make_mingw "librdkafka")

elif [ "$(uname -o)" == "GNU/Linux" ] ; then
    # Linux
    sudo apt-get install -y --install-recommends \
        zip unzip curl \
        gcc g++ cmake make \
        build-essential \
        pkg-config
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
    if ! [ -e /opt/vcpkg ] ; then
        mkdir -p /opt/vcpkg
        git clone https://github.com/microsoft/vcpkg.git /opt/vcpkg && /opt/vcpkg/bootstrap-vcpkg.sh
    fi
    PATH="/opt/vcpkg:${PATH}"
    cargo install cargo-vcpkg
    # NOTE: On MinGW64, you `vcpkg install librdkafka:x64-mingw-dynamic`
    #       On Linux, you `vcpkg install librdkafka:x64-linux`
    #       On Windows, you `vcpkg install librdkafka:x64-windows` BUT without VStudios installed, it WILL FAIL, so use mingw instead!
    vcpkg install librdkafka:x64-linux

    # odd cases where reinstall is needed
    sudo apt-get reinstall -y --install-recommends \
        librdkafka*
else
    # Windows or some other OS I care not about...
    echo "OS not supported"
    exit -1
fi

#cargo vcpkg build 
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