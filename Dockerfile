# the WASM build of the runtime is completely indepedent 
# we can avoid cache invalidations by running it in an extra container
# FIXME: We need to enfoce a specific nighlty version, since the mashnet node doesn't compile with the newest nightly. Nightlies before (and including) nightly-2020-05-14 are working.
FROM rustlang/rust@sha256:9ac425a47e25a7a5dac999362b89de2b91b21ce70c557a409c46280393f7b1f1 as wasm_builder

# install wasm toolchain for polkadot
RUN rustup target add wasm32-unknown-unknown --toolchain nightly

# Install wasm-gc. It's useful for stripping slimming down wasm binaries.
# (polkadot)
RUN cargo +nightly install --git https://github.com/alexcrichton/wasm-gc

# show backtraces
ENV RUST_BACKTRACE 1

## not sure which of theses ENV are actually needed for this step
#compiler ENV
ENV CC gcc
ENV CXX g++

#snapcraft ENV
ENV LC_ALL=C.UTF-8
ENV LANG=C.UTF-8

# Copy runtime library files
COPY ./runtime/Cargo.lock ./runtime/Cargo.toml ./runtime/
COPY ./runtime/src ./runtime/src
# Copy WASM build crate files
COPY ./runtime/wasm/build.sh ./runtime/wasm/Cargo.lock ./runtime/wasm/Cargo.toml ./runtime/wasm/
COPY ./runtime/wasm/src ./runtime/wasm/src

# get build script and build
COPY ./scripts/build.sh /scripts/build.sh
RUN /bin/bash /scripts/build.sh

# this container builds the mashnet-node binary from source files, the runtime library and the WASM file built previously
FROM rustlang/rust@sha256:9ac425a47e25a7a5dac999362b89de2b91b21ce70c557a409c46280393f7b1f1 as builder

WORKDIR /build

# install clang
RUN apt-get -y update && \
	apt-get install -y --no-install-recommends \
	clang

# show backtraces
ENV RUST_BACKTRACE 1

## not sure which of theses ENV are actually needed for this step
#compiler ENV
ENV CC gcc
ENV CXX g++

#snapcraft ENV
ENV LC_ALL=C.UTF-8
ENV LANG=C.UTF-8

# to avoid early cache invalidation, we build only dependencies first. For this we create fresh crates we are going to overwrite.
RUN USER=root cargo init --bin --name=mashnet-node
RUN USER=root cargo new --lib --name=mashnet-node-runtime runtime
# overwrite cargo.toml with real files
COPY Cargo.toml Cargo.lock build.rs ./
COPY ./runtime/Cargo.toml ./runtime/Cargo.lock ./runtime/

# build depedencies (and bogus source files)
RUN cargo build --release

# remove bogus build (but keep depedencies)
RUN cargo clean --release -p mashnet-node-runtime

# copy everything over (cache invalidation will happen here)
COPY . /build
# get wasm built in previous step
COPY --from=wasm_builder /runtime/wasm/target/wasm32-unknown-unknown/release ./runtime/wasm/target/wasm32-unknown-unknown/release
# build source again, dependencies are already built
RUN cargo build --release

# test
RUN cargo test --release -p mashnet-node-runtime


FROM debian:stretch

WORKDIR /runtime

RUN apt-get -y update && \
	apt-get install -y --no-install-recommends \
	openssl \
	curl \
	libssl-dev dnsutils

RUN mkdir -p /runtime/target/release/
COPY --from=builder /build/target/release/mashnet-node ./target/release/mashnet-node
COPY --from=builder /build/start-node.sh ./start-node.sh
COPY --from=builder /build/chainspec.json ./chainspec.json

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
