#!/bin/bash

# Development script for Docker Compose

case "$1" in
  up)
    docker-compose up -d
    echo "Fleetform services started!"
    echo "Web UI: http://localhost:3001"
    ;;
  down)
    docker-compose down
    ;;
  logs)
    docker-compose logs -f ${2:-fleetform-web}
    ;;
  cli)
    docker-compose exec fleetform-cli /workspace/target/release/fleetform ${@:2}
    ;;
  build)
    docker-compose build
    ;;
  *)
    echo "Usage: $0 {up|down|logs|cli|build}"
    echo "  up    - Start all services"
    echo "  down  - Stop all services"  
    echo "  logs  - View logs (optional service name)"
    echo "  cli   - Run fleetform CLI commands"
    echo "  build - Rebuild containers"
    ;;
esac