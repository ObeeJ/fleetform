package handlers

import (
	"encoding/json"
	"github.com/gofiber/fiber/v2"
	"io/ioutil"
)

func UIHandler(c *fiber.Ctx) error {
	// Try to read plan data from file first
	data, err := ioutil.ReadFile("../fleetform_plan.json")
	if err == nil {
		var planData map[string]interface{}
		if json.Unmarshal(data, &planData) == nil {
			return c.JSON(planData)
		}
	}
	
	// Fallback to default data
	defaultData := fiber.Map{
		"plan": []string{
			"Create: aws_instance.example",
			"Update: aws_s3_bucket.my_bucket",
		},
		"status": "Planning...",
	}
	return c.JSON(defaultData)
}