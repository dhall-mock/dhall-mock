FROM ubuntu:latest

COPY dhall-mock-server-x86_64-unknown-linux-gnu/main main

RUN chmod +x main

EXPOSE 8088/tcp
EXPOSE 8089/tcp

ENTRYPOINT ["./main"]
