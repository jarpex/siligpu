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

check-licenses:
    cargo deny check licenses bans sources

check-advisories:
    cargo deny check advisories

compliance: check-licenses check-advisories licenses