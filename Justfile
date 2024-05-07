build-small:
    #!/bin/sh
    RUSTFLAGS="-Zlocation-detail=none" cargo +nightly build -Z build-std=std,panic_abort --release --target x86_64-unknown-linux-gnu

release: clean build-small
    rm -f ./stringbar
    upx --best --lzma target/x86_64-unknown-linux-gnu/release/stringbar -o ./stringbar

clean:
    #!/bin/sh
    cargo clean
    rm -f ./stringbar
