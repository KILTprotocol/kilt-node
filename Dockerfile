FROM ubuntu:xenial

WORKDIR /substrate

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

COPY . /substrate

RUN /bin/bash build.sh

RUN cargo build && cargo test

EXPOSE 30333 9933 9944

RUN ls -la .

# boot node for Alice:
# ./target/debug/node --chain local --key Alice --name "ALICE" --node-key 0000000000000000000000000000000000000000000000000000000000000001 --validator --telemetry-url ws://telemetry-backend.kilt-prototype.tk:1024
# Alice's address: /ip4/0.0.0.0/tcp/30333/p2p/QmQZ8TjTqeDj3ciwr93EJ95hxfDsb9pEYDizUAbWpigtQN

# boot node for Bob:
# ./target/debug/node --chain local --key Bob --name "BOB" --node-key 0000000000000000000000000000000000000000000000000000000000000002 --validator --telemetry-url ws://telemetry-backend.kilt-prototype.tk:1024
# Bobs address: /ip4/0.0.0.0/tcp/30333/p2p/QmXiB3jqqn2rpiKU7k1h7NJYeBg8WNSx9DiTRKz9ti2KSK

CMD ["./target/debug/node", "--chain", "local", "--key", "Alice", "--name", "\"ALICE\"", "--node-key", "0000000000000000000000000000000000000000000000000000000000000001", "--validator"]