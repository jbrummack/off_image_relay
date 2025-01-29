#ARG APP_NAME=app

FROM rust:1.81 AS builder
WORKDIR /usr/src/$APP_NAME

RUN apt-get update; \
    apt-get install -y --no-install-recommends \
    libclang-dev
RUN rm -rf /var/lib/apt/lists/*

RUN cargo init --bin
COPY Cargo.toml ./
#COPY Cargo.lock ./

RUN cargo build --release
RUN rm ./src/*.rs ./target/release/deps/app*

# build
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/app /usr/local/bin/app
#ENV APP_NAME $APP_NAME

EXPOSE 8080

CMD app

#RUN USER=root cargo new --bin app
#WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files first (to leverage Docker's caching)

#COPY src ./src

#RUN LIBCLANG_PATH=/usr/lib/llvm-14/lib/ cargo build --release
#RUN rm src/*.rs

#COPY src ./src

# Build for release
#RUN rm ./target/release/deps/app*
#RUN cargo build --release


#EXPOSE 8080

# Add this to keep the builder stage running
#CMD ["sh", "-c", "ls -R /usr/src/app/app/target/release && /bin/bash"]
#CMD ["/usr/src/app/target/release/off_image_relay"]
