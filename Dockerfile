# Stage 1: Builder (ใช้ภาพที่มีเครื่องมือครบเพื่อคอมไพล์)
FROM rust:1.85-slim AS builder

# Install dependencies for OpenSSL
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY . .

# คอมไพล์แบบ Release เพื่อความแรงสูงสุด
RUN cargo build --release

# Stage 2: Runner (ใช้ภาพที่เล็กและปลอดภัยที่สุด)
# เราใช้ Google Distroless เพราะไม่มี Shell ให้ Hacker โจมตี
FROM gcr.io/distroless/cc-debian12

WORKDIR /app
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /usr/src/app/target/release/iron-mask-proxy .
COPY ./config.yaml .

# ตั้งค่า ENV พื้นฐาน
ENV TARGET_URL=http://localhost:8080
EXPOSE 3000

CMD ["./iron-mask-proxy"]
