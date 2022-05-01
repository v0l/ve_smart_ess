ARG IMAGE=rust:latest

#Builder
FROM $IMAGE as build
WORKDIR /app/src
COPY . .
RUN cargo install --path . --root /app/build

# Runner
FROM $IMAGE as runner
WORKDIR /app
COPY --from=build /app/build .
COPY smart_ess.json .

ENTRYPOINT ["/app/bin/ve_smart_ess"]