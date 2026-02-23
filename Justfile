[working-directory: 'ventrica_ffi']
build-ffi-native:
  cargo build --release

xcframework: apple-build
  rm -rf ventrica_ffi/_build/libVentrica.xcframework
  rm -rf ventrica_ffi/_build/libs
  mkdir ventrica_ffi/_build/libs
  lipo -create -output ventrica_ffi/_build/libs/ventrica-macos.a \
    target/aarch64-apple-darwin/release/libventrica_ffi.a \
    # target/x86_64-apple-darwin/release/libventrica_ffi.a

  xcodebuild -create-xcframework \
    -library ventrica_ffi/_build/libs/ventrica-macos.a -headers ventrica_ffi/_build/include \
    -output ventrica_ffi/_build/libVentrica.xcframework

[working-directory: 'ventrica_ffi']
apple-build:
  # macOS (native)
  cargo build --release --target aarch64-apple-darwin
  # cargo build --release --target x86_64-apple-darwin