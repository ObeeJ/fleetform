package handlers

import (
	"github.com/gofiber/fiber/v2"
)

func ConfigHandler(c *fiber.Ctx) error {
	configs := []string{"fleetform.hcl", "fleetform.json", "fleetform.yaml"}
	return c.JSON(fiber.Map{
		"message": "Fleetform configuration endpoint",
		"configs": configs,
	})
}