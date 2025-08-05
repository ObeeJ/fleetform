package main

import (
	"github.com/ObeeJ/fleetform/fiber/handlers"
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/websocket/v2"
	"log"
)

func main() {
	app := fiber.New()

	app.Static("/", "./static")
	app.Get("/ui", handlers.UIHandler)
	app.Get("/config", handlers.ConfigHandler)
	app.Get("/state", handlers.StateHandler)
	app.Get("/modules", handlers.ModulesHandler)
	app.Get("/diff", handlers.DiffHandler)
	app.Get("/realtime", websocket.New(handlers.RealtimeHandler))

	log.Fatal(app.Listen(":3001"))
}