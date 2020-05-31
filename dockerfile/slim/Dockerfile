
# ------------------------------------------------------------------------------
# Packager Stage
# ------------------------------------------------------------------------------


FROM debian:10 as packager

ARG BINARIE_PATH=target/release/dhall_mock_server

WORKDIR /opt/dhall-mock
COPY $BINARIE_PATH .

RUN addgroup -gid 1000 dhall-mock
RUN adduser --system --disabled-login --disabled-password -uid 10001 --ingroup dhall-mock dhall-mock

RUN chmod +x dhall_mock_server
RUN chown dhall-mock:dhall-mock dhall_mock_server


# ------------------------------------------------------------------------------
# Release Stage
# ------------------------------------------------------------------------------

FROM gcr.io/distroless/cc-debian10

WORKDIR /opt/dhall-mock

COPY --from=packager /opt/dhall-mock/dhall_mock_server .
COPY --from=packager /etc/passwd /etc/passwd
USER dhall-mock

ENTRYPOINT ["./dhall_mock_server"]
EXPOSE 8088/tcp

