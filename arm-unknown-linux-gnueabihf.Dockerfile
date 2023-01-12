FROM ghcr.io/cross-rs/arm-unknown-linux-gnueabihf:0.2.4

# Get Raspberry Pi cross-compiler tools
RUN apt-get update && apt-get install -y gcc-arm-linux-gnueabihf

# Download OpenSSL
RUN curl https://www.openssl.org/source/openssl-1.1.1s.tar.gz -o /tmp/openssl-1.1.1s.tar.gz &&\
    tar xzf /tmp/openssl-1.1.1s.tar.gz -C /tmp


# Compile OpenSSL
RUN cd /tmp/openssl-1.1.1s &&\
    ./Configure linux-generic32 shared\
    --cross-compile-prefix=arm-linux-gnueabihf- &&\
    make depend && make && make install