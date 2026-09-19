FROM golang:1.24-alpine AS builder

WORKDIR /app
ENV GOPROXY=https://proxy.golang.org,direct
ENV GOSUMDB=sum.golang.org
ENV CGO_ENABLED=0
ENV GOTOOLCHAIN=auto

COPY go.mod go.sum* ./
RUN go mod download

COPY . .
RUN go build -o main .

FROM alpine:3.20
RUN apk --no-cache add ca-certificates wget \
    && adduser -D -H -u 10001 fleetform \
    && mkdir -p /data && chown fleetform:fleetform /data
WORKDIR /app

COPY --from=builder /app/main .
COPY --from=builder /app/static ./static

ENV FLEETFORM_DB=/data/fleetform.db
USER fleetform
EXPOSE 3001
CMD ["./main"]
