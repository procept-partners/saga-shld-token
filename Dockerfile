FROM rust:latest

# Set the working directory inside the container
WORKDIR /shld

# Install wasm32 target for compiling to WebAssembly
RUN rustup target add wasm32-unknown-unknown

# Copy the Cargo.toml and Cargo.lock to the working directory
COPY Cargo.toml Cargo.lock ./

# Copy the source code files to the working directory
COPY src/ ./src/

# Copy the tests directory to the working directory
COPY tests/ ./tests/

# Build the project dependencies first to cache them
RUN cargo build --release

# Copy the rest of the application code
COPY . .

# Final build command for the application
RUN cargo build --release

