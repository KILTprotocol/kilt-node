# this container builds the mashnet-node binary from source files and the runtime library
# pinned the version to avoid build cache invalidation
FROM paritytech/ci-linux:5297d82c-20201107 as builder

WORKDIR /build
# to avoid early cache invalidation, we build only dependencies first. For this we create fresh crates we are going to overwrite.

# create runtime & nodes
RUN USER=root cargo new --bin --name=kilt-parachain nodes/parachain
RUN USER=root cargo new --bin --name=mashnet-node nodes/standalone
RUN USER=root cargo new --lib --name=mashnet-node-runtime runtimes/standalone
RUN USER=root cargo new --lib --name=kilt-parachain-runtime runtimes/parachain
RUN USER=root cargo new --lib --name=kilt-parachain-primitives primitives

# overwrite cargo.toml with real files
COPY Cargo.toml Cargo.lock ./
COPY ./runtimes/standalone/Cargo.toml ./runtimes/standalone/
COPY ./runtimes/parachain/Cargo.toml ./runtimes/parachain/
COPY ./nodes/standalone/Cargo.toml ./nodes/standalone/
COPY ./nodes/parachain/Cargo.toml ./nodes/parachain/
COPY ./primitives/Cargo.toml ./primitives/

COPY ./runtimes/standalone/build.rs ./runtimes/standalone/
COPY ./runtimes/parachain/build.rs ./runtimes/parachain/
COPY ./nodes/standalone/build.rs ./nodes/standalone/
COPY ./nodes/parachain/build.rs ./nodes/parachain/

# pallets
RUN USER=root cargo new --lib --name=pallet-attestation pallets/attestation
RUN USER=root cargo new --lib --name=pallet-ctype pallets/ctype
RUN USER=root cargo new --lib --name=pallet-delegation pallets/delegation
RUN USER=root cargo new --lib --name=pallet-did pallets/did
RUN USER=root cargo new --lib --name=pallet-error pallets/error
RUN USER=root cargo new --lib --name=pallet-portablegabi pallets/portablegabi
COPY ./pallets/attestation/Cargo.toml ./pallets/attestation/
COPY ./pallets/ctype/Cargo.toml ./pallets/ctype/
COPY ./pallets/delegation/Cargo.toml ./pallets/delegation/
COPY ./pallets/did/Cargo.toml ./pallets/did/
COPY ./pallets/error/Cargo.toml ./pallets/error/
COPY ./pallets/portablegabi/Cargo.toml ./pallets/portablegabi/

# build depedencies (and bogus source files)
RUN cargo build --release

# remove bogus build (but keep dependencies)
RUN cargo clean --release -p mashnet-node-runtime
RUN cargo clean --release -p ctype
RUN cargo clean --release -p delegation
RUN cargo clean --release -p did
RUN cargo clean --release -p error
RUN cargo clean --release -p portablegabi

# copy everything over (cache invalidation will happen here)
COPY . /build
# build source again, dependencies are already built

# test
RUN cargo test --release --all

ARG NODE_TYPE=mashnet-node

RUN cargo build --release -p $NODE_TYPE


FROM debian:stretch-slim

WORKDIR /runtime

RUN apt-get -y update && \
	apt-get install -y --no-install-recommends \
	openssl \
	curl \
	libssl-dev dnsutils

# cleanup linux dependencies
RUN apt-get autoremove -y
RUN apt-get clean -y
RUN rm -rf /tmp/* /var/tmp/*

RUN mkdir -p /runtime/target/release/
COPY --from=builder /build/target/release/$NODE_TYPE ./target/release/$NODE_TYPE
COPY --from=builder /build/start-node.sh ./start-node.sh
COPY --from=builder /build/chainspec.json ./chainspec.json
COPY --from=builder /build/chainspec-devnet.json ./chainspec-devnet.json

RUN chmod a+x *.sh
RUN ls -la .

# expose node ports
EXPOSE 30333 9933 9944

#
# Pass the node start command to the docker run command
#
# To start full node:
# ./start-node --telemetry
#
# To start a full node that connects to Alice:
# ./start-node.sh --connect-to Alice -t
#
CMD ["echo","\"Please provide a startup command.\""]
