FROM pitkley/rust:stable

ENV PATH "/root/.cargo/bin:$PATH"
RUN \
    # Install cargo binaries
    cargo install --vers 0.10.0 rustfmt

RUN \
    # Install required packages
    apt-get update \
    && apt-get install -y --no-install-recommends \
        make \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
