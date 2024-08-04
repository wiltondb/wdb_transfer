@echo on

pushd .. || exit /b 1
git clone --branch sspi_response_header_type https://github.com/wiltondb/tiberius.git || exit /b 1
popd || exit /b 1

rustup install 1.70.0 || exit /b 1
cargo +1.70.0 build --release || exit /b 1
