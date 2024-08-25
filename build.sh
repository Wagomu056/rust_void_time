rm -rfv output/
mkdir output

# create header
cbindgen --crate rust_void_time --output output/include/rust_void_time.h

# create .a files
mkdir output/aarch64-apple-ios
cargo build --release --target aarch64-apple-ios
cp target/aarch64-apple-ios/release/librust_void_time.a output/aarch64-apple-ios/

mkdir output/aarch64-apple-ios-sim
cargo build --release --target aarch64-apple-ios-sim
cp target/aarch64-apple-ios-sim/release/librust_void_time.a output/aarch64-apple-ios-sim

# create xcframework
xcodebuild -create-xcframework \
    -library output/aarch64-apple-ios/librust_void_time.a -headers output/include \
    -library output/aarch64-apple-ios-sim/librust_void_time.a -headers output/include \
    -output output/rust_void_time.xcframework
