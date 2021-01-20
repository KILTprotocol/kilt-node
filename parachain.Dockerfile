# this container builds the kilt-parachain binary from source files and the runtime library
# pinned the version to avoid build cache invalidation
FROM paritytech/ci-linux:5297d82c-20201107 as builder

WORKDIR /build

COPY . /build
RUN cargo build --release -p kilt-parachain
RUN cargo test --release -p kilt-parachain

# ===== SECOND STAGE ======

FROM debian:buster-slim
LABEL description="This is the 2nd stage: a very small image where we copy the kilt-parachain binary."

COPY --from=builder /build/target/release/kilt-parachain /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /node node && \
	mkdir -p /node/.local/share/node && \
	chown -R node:node /node/.local && \
	ln -s /node/.local/share/node /data && \
	rm -rf /usr/bin /usr/sbin

COPY --from=builder /build/target/release/wbuild/kilt-parachain-runtime/kilt_parachain_runtime.compact.wasm /node/parachain.wasm

USER node
EXPOSE 30333 9933 9944
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/kilt-parachain"]
CMD ["--help"]
