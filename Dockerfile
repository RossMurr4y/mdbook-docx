FROM rossmurr4y/rust-builder-base AS builder
ARG CRATE="mdbook"
ARG MDBOOK_VERSION="0.4.7"
COPY . ./
RUN cargo install mdbook --vers ${MDBOOK_VERSION} --verbose; \
    cargo install --path .


FROM pandoc/ubuntu-latex AS mdbook-docx
COPY --from=builder /usr/local/cargo/bin/mdbook* /usr/bin/
RUN apt-get update --allow-insecure-repositories; \
    apt-get install --no-install-recommends -y \
        ca-certificates \
        && rm -rf /var/cache/apt/lists
SHELL ["/bin/bash"]
WORKDIR /book
ENTRYPOINT [ "/usr/bin/mdbook" ]