package handlers

import (
	"encoding/json"

	"github.com/gofiber/fiber/v2"
)

func StateHandler(c *fiber.Ctx) error {
	data, path, err := readFirst(
		"../.fleetform/state.json",
		".fleetform/state.json",
		"/data/.fleetform/state.json",
	)
	if err != nil {
		return c.JSON(fiber.Map{
			"version":   1,
			"serial":    0,
			"resources": []string{},
			"managed":   []any{},
			"note":      "no state file yet; run fleetform apply",
		})
	}
	var stateData map[string]interface{}
	if json.Unmarshal(data, &stateData) != nil {
		return c.Status(500).JSON(fiber.Map{"error": "Failed to parse state data", "path": path})
	}
	stateData["source"] = path
	return c.JSON(stateData)
}
