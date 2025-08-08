@echo off

if "%1"=="up" (
    docker-compose up -d
    echo Fleetform services started!
    echo Web UI: http://localhost:3001
) else if "%1"=="down" (
    docker-compose down
) else if "%1"=="logs" (
    if "%2"=="" (
        docker-compose logs -f fleetform-web
    ) else (
        docker-compose logs -f %2
    )
) else if "%1"=="cli" (
    docker-compose exec fleetform-cli /workspace/target/release/fleetform %*
) else if "%1"=="build" (
    docker-compose build
) else (
    echo Usage: %0 {up^|down^|logs^|cli^|build}
    echo   up    - Start all services
    echo   down  - Stop all services
    echo   logs  - View logs ^(optional service name^)
    echo   cli   - Run fleetform CLI commands
    echo   build - Rebuild containers
)