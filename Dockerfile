FROM ubuntu:xenial as builder

WORKDIR /build

# install tools and dependencies
RUN apt -y update && \
  apt install -y --no-install-recommends \
	software-properties-common curl git file binutils binutils-dev snapcraft \
	make cmake ca-certificates g++ zip dpkg-dev python rhash rpm openssl gettext\
  build-essential pkg-config libssl-dev libudev-dev ruby-dev time

#install nodejs
RUN curl -sL https://deb.nodesource.com/setup_8.x | sudo -E bash - && \
  apt-get install -y nodejs

# install rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# rustup directory
ENV PATH /root/.cargo/bin:$PATH

# setup rust beta and nightly channel's
RUN rustup install beta
RUN rustup install nightly

# install wasm toolchain for polkadot
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
# Install wasm-gc. It's useful for stripping slimming down wasm binaries.
# (polkadot)
RUN cargo +nightly install --git https://github.com/alexcrichton/wasm-gc

# setup default stable channel
RUN rustup default nightly

# show backtraces
ENV RUST_BACKTRACE 1

# cleanup
RUN apt autoremove -y
RUN apt clean -y
RUN rm -rf /tmp/* /var/tmp/*

#compiler ENV
ENV CC gcc
ENV CXX g++

#snapcraft ENV
ENV LC_ALL=C.UTF-8
ENV LANG=C.UTF-8

COPY . /build

RUN /bin/bash build.sh

RUN cargo build && cargo test


FROM ubuntu:xenial

WORKDIR /runtime
RUN mkdir -p /runtime/target/debug/
COPY --from=builder /build/target/debug/node ./target/debug/node
COPY --from=builder /build/scripts/kilt-node-testnet.sh ./kilt-node-testnet.sh
COPY --from=builder /build/scripts/lookup-master-bootnode-testnet.sh ./lookup-master-bootnode-testnet.sh
COPY --from=builder /build/scripts/kilt-master-bootnode-testnet.sh ./kilt-master-bootnode-testnet.sh

RUN chmod a+x *.sh
RUN ls -la .

# expose node ports
EXPOSE 30333 9933 9944

#
# Pass the node start command to the docker run command
#
# To start a master boot node (no initial connection to other nodes):
# ./kilt-master-bootnode-testnet.sh --key Alice --name "ALICE" --node-key 0000000000000000000000000000000000000000000000000000000000000001
#
# To start a node that connects to the master bootnode:
# ./kilt-node-testnet.sh --key Charly --name "CHARLY"
#
CMD ["echo","\"Please provide a startup command.\""]
