FROM ubuntu:xenial as builder

WORKDIR /build

# install tools and dependencies
RUN apt-get -y update && \
	apt-get install -y --no-install-recommends \
	software-properties-common curl git file binutils binutils-dev snapcraft \
	make cmake ca-certificates g++ zip dpkg-dev python rhash rpm openssl gettext\
	build-essential pkg-config libssl-dev libudev-dev ruby-dev time clang

# install rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal

# rustup directory
ENV PATH /root/.cargo/bin:$PATH

# We only want specific components
RUN rustup set profile minimal

# setup rust beta and nightly channel's
RUN rustup install nightly
RUN rustup install stable

# install wasm toolchain for polkadot
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
RUN rustup component add clippy

# Install wasm-gc. It's useful for stripping slimming down wasm binaries.
# (polkadot)
RUN cargo +nightly install --git https://github.com/alexcrichton/wasm-gc

# show backtraces
ENV RUST_BACKTRACE 1

# cleanup
RUN apt-get autoremove -y
RUN apt-get clean -y
RUN rm -rf /tmp/* /var/tmp/*

#compiler ENV
ENV CC gcc
ENV CXX g++

#snapcraft ENV
ENV LC_ALL=C.UTF-8
ENV LANG=C.UTF-8

COPY . /build

RUN /bin/bash scripts/build.sh

RUN cargo build --release
RUN cargo test --release -p mashnet-node-runtime
RUN cargo clippy -- -D warnings


FROM ubuntu:xenial

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
COPY --from=builder /build/chainspec.devnet.json ./chainspec.devnet.json

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
