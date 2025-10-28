FROM rust:latest AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*
RUN git clone https://github.com/AstroX11/word-vaildator-api .
RUN cargo build --release

CMD ["./target/release/word_validator_api"]
