FROM golang:1.24-alpine AS builder

WORKDIR /app
ENV GOPROXY=https://proxy.golang.org,direct
ENV GOSUMDB=sum.golang.org
ENV CGO_ENABLED=0

COPY go.mod go.sum ./
RUN go mod download -x

COPY . .
RUN go build -o main .

FROM alpine:latest
RUN apk --no-cache add ca-certificates
WORKDIR /root/

COPY --from=builder /app/main .
COPY --from=builder /app/static ./static

EXPOSE 3001
CMD ["./main"]