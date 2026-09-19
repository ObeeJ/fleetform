package main

import (
	"log"
	"os"
	"strings"
	"time"

	"github.com/ObeeJ/fleetform/fiber/internal/httpapi"
	"github.com/ObeeJ/fleetform/fiber/internal/store"
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/helmet"
	"github.com/gofiber/fiber/v2/middleware/limiter"
	"github.com/gofiber/fiber/v2/middleware/logger"
	"github.com/gofiber/fiber/v2/middleware/recover"
)

func main() {
	dbPath := os.Getenv("FLEETFORM_DB")
	if dbPath == "" {
		dbPath = "fleetform.db"
	}
	st, err := store.Open(dbPath)
	if err != nil {
		log.Fatal(err)
	}

	app := fiber.New(fiber.Config{
		AppName:               "Fleetform",
		DisableStartupMessage: false,
		ErrorHandler: func(c *fiber.Ctx, err error) error {
			code := fiber.StatusInternalServerError
			if e, ok := err.(*fiber.Error); ok {
				code = e.Code
			}
			return c.Status(code).JSON(fiber.Map{"error": "request_failed", "message": "The request could not be completed."})
		},
	})
	app.Use(recover.New())
	app.Use(logger.New())
	app.Use(helmet.New())
	app.Use(limiter.New(limiter.Config{
		Max:        30,
		Expiration: time.Minute,
		Next: func(c *fiber.Ctx) bool {
			return !strings.HasPrefix(c.Path(), "/api/auth")
		},
	}))

	httpapi.New(st).Register(app)

	app.Static("/", "./static", fiber.Static{Compress: true, Index: "index.html"})
	app.Get("/*", func(c *fiber.Ctx) error {
		if strings.HasPrefix(c.Path(), "/api") {
			return c.Status(404).JSON(fiber.Map{"error": "not_found", "message": "Unknown API route."})
		}
		return c.SendFile("./static/index.html")
	})

	addr := os.Getenv("FLEETFORM_ADDR")
	if addr == "" {
		addr = ":3001"
	}
	log.Fatal(app.Listen(addr))
}
