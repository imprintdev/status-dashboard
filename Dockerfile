FROM debian:trixie-slim

RUN apt-get update
RUN apt-get install -y libssl-dev openssl ca-certificates ssh nginx libsqlite3-0
WORKDIR /app
COPY status-dashboard .
RUN ls -a
RUN ls -a dist
COPY dist /usr/share/nginx/html

ENTRYPOINT [ "/app/status-dashboard" ]