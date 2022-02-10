FROM rust:1.54.0-slim-bullseye AS builder

ARG MDBOOK_VERSION="0.4.7"

COPY . ./

RUN apt-get update --allow-insecure-repositories; \
    apt-get install -y \
        libssl-dev \
        pkg-config \
        ca-certificates \
        build-essential \
        make \
        perl \
        gcc \
        libc6-dev; \
    dpkgArch="$(dpkg --print-architecture)"; \
    echo "Arch: ${dpkgArch}"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu' ;; \
        i386) rustArch='i686-unknown-linux-gnu' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    echo "Setting cargo default arch to: ${rustArch}"; \
    rustup set default-host ${rustArch}; \
    cargo install --path . ; \
    cargo install mdbook --vers ${MDBOOK_VERSION} --verbose

FROM pandoc/ubuntu-latex AS mdbook-docx
COPY --from=builder /usr/local/cargo/bin/mdbook* /usr/bin/
RUN apt-get update --allow-insecure-repositories; \
    apt-get install --no-install-recommends -y \
        ca-certificates \
        texlive-xetex \
        graphviz \
        plantuml \
        && rm -rf /var/cache/apt/lists
SHELL ["/bin/bash"]
WORKDIR /book
ENTRYPOINT [ "/usr/bin/mdbook" ]