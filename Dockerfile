# this container builds the kilt-parachain binary from source files and the runtime library
# pinned the version to avoid build cache invalidation

# c4f0daf3-20220110 -> rust version rustc 1.58.0-nightly (46b8e7488 2021-11-07)
FROM paritytech/ci-linux:f203f298-20220421 as builder

WORKDIR /build

ARG FEATURES=default

COPY . .

RUN cargo build --locked --release --features $FEATURES

# ===== SECOND STAGE ======

FROM docker.io/library/ubuntu:20.04
LABEL description="This is the 2nd stage: a very small image where we copy the kilt-parachain binary."

ARG NODE_TYPE=kilt-parachain

COPY --from=builder /build/target/release/$NODE_TYPE /usr/local/bin/node-executable

RUN useradd -m -u 1000 -U -s /bin/sh -d /node node && \
	mkdir -p /node/.local/share/node && \
	chown -R node:node /node/.local && \
	ln -s /node/.local/share/node /data && \
	rm -rf /usr/bin /usr/sbin

USER node
EXPOSE 30333 9933 9944
VOLUME ["/data"]

COPY ./dev-specs /node/dev-specs

ENTRYPOINT ["/usr/local/bin/node-executable"]
CMD ["--help"]
