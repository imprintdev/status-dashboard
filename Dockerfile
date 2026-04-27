FROM debian:trixie-slim

RUN apt-get update
RUN apt-get install -y libssl-dev openssl ca-certificates ssh nginx libsqlite3-0
WORKDIR /app
RUN ls -a .
RUN ls -a /var/www/
RUN ls -a /var/www/html
RUN ls -a dist
COPY dist /var/www/html
COPY status-dashboard .

ENTRYPOINT [ "/app/status-dashboard" ]