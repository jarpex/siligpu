default:
    @just --list

build-clean:
    ./build-clean.sh

build:
    cargo build --release

licenses:
    cargo about generate about.hbs -o THIRD_PARTY_LICENSES.html

clean-licenses:
    rm -f THIRD_PARTY_LICENSES.html

compliance: 
    licenses