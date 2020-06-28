#!/bin/bash

CUR_DIR="$(pwd)"

echo 'Requesting superuser access'
function refresh_sudo_access {
    sudo -v
}
refresh_sudo_access

echo 'Building dir structure'
rm -rf $CUR_DIR/test
mkdir -p $CUR_DIR/test/a1 $CUR_DIR/test/b1 $CUR_DIR/test/a2 $CUR_DIR/test/b1 $CUR_DIR/test/b2

echo 'Generating directory A'
for i in {1..7262}
do
    NAME=$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 32 | head -n 1)
    dd if=/dev/urandom of=$CUR_DIR/test/a1/$NAME bs=1k count=13 2> /dev/null
done
refresh_sudo_access

echo 'Generating directory B'
for i in {1..252}
do
    NAME=$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 32 | head -n 1)
    dd if=/dev/urandom of=$CUR_DIR/test/b1/$NAME bs=1k count=4161 2> /dev/null
done
refresh_sudo_access

echo 'Building latest lms binary'
cargo build --release
refresh_sudo_access

DROP_CACHE_CMD='sync; echo 3 | sudo tee /proc/sys/vm/drop_caches'
DROP_CACHE_REFILL_DIRS_CMD="rsync -r --delete $CUR_DIR/test/a1/ $CUR_DIR/test/a2/ ; rsync -r --delete $CUR_DIR/test/b1/ $CUR_DIR/test/b2/ ; $DROP_CACHE_CMD"

echo 'Starting test runs ...'
function clear_test_dirs {
    rm -rf $CUR_DIR/a2/*
    rm -rf $CUR_DIR/b2/*
}

NEW_LMS="$CUR_DIR/target/release/lms"
SYS_LMS="/home/$(whoami)/.cargo/bin/lms"

function run_benchmark {
    SRC_DIR="$1"
    DST_DIR="$2"

    echo "Testing sync $SRC_DIR --> $DST_DIR"
    hyperfine --prepare "$DROP_CACHE_CMD" "$NEW_LMS sync $SRC_DIR $DST_DIR"
    clear_test_dirs
    hyperfine --prepare "$DROP_CACHE_CMD" "$SYS_LMS sync $SRC_DIR $DST_DIR"
    clear_test_dirs
    hyperfine --prepare "$DROP_CACHE_CMD" "rsync -r --delete $SRC_DIR $DST_DIR"
    clear_test_dirs

    echo "Testing cp $SRC_DIR --> $DST_DIR"
    hyperfine --prepare "$DROP_CACHE_CMD" "$NEW_LMS cp $SRC_DIR $DST_DIR"
    clear_test_dirs
    hyperfine --prepare "$DROP_CACHE_CMD" "$SYS_LMS cp $SRC_DIR $DST_DIR"
    clear_test_dirs
    hyperfine --prepare "$DROP_CACHE_CMD" "cp -r $SRC_DIR $DST_DIR"
    clear_test_dirs

    echo "Testing rm $SRC_DIR --> $DST_DIR"
    hyperfine --prepare "$DROP_CACHE_REFILL_DIRS_CMD" "$NEW_LMS rm $DST_DIR"
    hyperfine --prepare "$DROP_CACHE_REFILL_DIRS_CMD" "$SYS_LMS rm $DST_DIR"
    hyperfine --prepare "$DROP_CACHE_REFILL_DIRS_CMD" "rm -rf $DST_DIR*"
}

run_benchmark "$CUR_DIR/test/a1/" "$CUR_DIR/test/a2/"
run_benchmark "$CUR_DIR/test/b1/" "$CUR_DIR/test/b2/"
