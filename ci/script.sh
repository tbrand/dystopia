# This script takes care of testing your crate

set -ex

main() {
    export LD_LIBRARY_PATH=${LD_LIBRARY_PATH}:/usr/lib/postgresql/9.5/lib

    cross build --target $TARGET --features all
    cross build --target $TARGET --features all --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET --features all
    cross test --target $TARGET --features all --release
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
