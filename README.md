
## Mac os
> brew install coreutils

## Make chain spec file
> ./target/debug/appchain-atocha build-spec --disable-default-bootnode --chain dev > atochaSpec.json
> ./target/release/appchain-atocha build-spec --disable-default-bootnode --chain dev > atochaSpec.json
> sha256sum atochaSpec.json > SHA256SUMS

## Start a test net.
> ./target/debug/appchain-atocha --ws-external --rpc-external --rpc-cors=all --prometheus-external --alice --chain local --base-path /tmp/alice
> ./target/debug/appchain-atocha --dev --rpc-external --rpc-cors=all --tmp

## Update test-net
> cargo update -p pallet-octopus-appchain
