FROM rust:1.91-slim-bookworm

WORKDIR /usr/src/pdfsnap-server

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static
COPY build.rs ./

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    build-essential

EXPOSE 6970/tcp

CMD ["cargo", "run", "--bin", "pdfsnap-server"]
