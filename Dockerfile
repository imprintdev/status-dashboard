FROM debian:trixie-slim

RUN apt-get update
RUN apt-get install -y libssl-dev openssl ca-certificates ssh nginx libsqlite3-0
WORKDIR /app
COPY dist /var/www/html
COPY status-dashboard .

ENTRYPOINT [ "/app/status-dashboard" ]