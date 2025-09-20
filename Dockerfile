# Etapa de build
FROM rust:1.85-alpine AS builder

# Instala dependencias necesarias para compilar (musl-dev para alpine)
RUN apk add --no-cache musl-dev

# Crea un directorio de trabajo
WORKDIR /app

# Copia los manifests de Cargo primero para cachear dependencias
COPY Cargo.toml Cargo.lock ./

# Construye dependencias (dummy src para cache)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copia el código real
COPY src src

# Limpia el target anterior para forzar recompilación con el código real
RUN cargo clean

# Construye la app final (ahora con el código real)
RUN cargo build --release

# Etapa final: imagen runtime ligera
FROM alpine:3.20

# Instala libgcc para compatibilidad con binarios Rust
RUN apk add --no-cache libgcc

# Copia el binario desde la etapa de build
COPY --from=builder /app/target/release/my-arbolitos /usr/local/bin/my-arbolitos

# Entry point: ejecuta la CLI
ENTRYPOINT ["my-arbolitos"]