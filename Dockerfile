FROM rust:1.82
WORKDIR /usr/src/mars-api
RUN mkdir /app
COPY . .
RUN cargo install --path . --root /app
ENV PATH="$PATH:/app/bin"
CMD ["mars_api_rs"]
