package handlers

import (
	"encoding/json"

	"github.com/gofiber/fiber/v2"
)

func UIHandler(c *fiber.Ctx) error {
	data, path, err := readFirst(
		"../fleetform_plan.json",
		"fleetform_plan.json",
		"/data/fleetform_plan.json",
	)
	if err != nil {
		return c.JSON(fiber.Map{
			"changes": []any{},
			"add":     0,
			"change":  0,
			"destroy": 0,
			"live":    false,
			"note":    "no plan file yet; run fleetform plan",
		})
	}
	var planData map[string]interface{}
	if json.Unmarshal(data, &planData) != nil {
		return c.Status(500).JSON(fiber.Map{"error": "Failed to parse plan", "path": path})
	}
	planData["source"] = path
	return c.JSON(planData)
}
