package handlers

import (
	"encoding/json"
	"github.com/gofiber/fiber/v2"
	"io/ioutil"
)

func StateHandler(c *fiber.Ctx) error {
	// Try to read state data from file
	data, err := ioutil.ReadFile("../.fleetform/state.json")
	if err != nil {
		return c.Status(500).JSON(fiber.Map{"error": "Failed to read state data"})
	}
	
	var stateData map[string]interface{}
	if json.Unmarshal(data, &stateData) != nil {
		return c.Status(500).JSON(fiber.Map{"error": "Failed to parse state data"})
	}
	
	return c.JSON(stateData)
}