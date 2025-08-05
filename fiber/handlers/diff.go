package handlers

import (
	"encoding/json"
	"github.com/gofiber/fiber/v2"
	"io/ioutil"
)

type DiffEntry struct {
	Action   string `json:"action"`
	Resource string `json:"resource"`
	Before   string `json:"before,omitempty"`
	After    string `json:"after,omitempty"`
}

func DiffHandler(c *fiber.Ctx) error {
	// Read current plan
	planData, err := ioutil.ReadFile("../fleetform_plan.json")
	if err != nil {
		return c.Status(500).JSON(fiber.Map{"error": "Failed to read plan"})
	}

	var plan map[string]interface{}
	json.Unmarshal(planData, &plan)

	// Generate diff view
	diff := []DiffEntry{
		{
			Action:   "create",
			Resource: "aws_instance.example",
			After:    "t3.micro instance",
		},
		{
			Action:   "update",
			Resource: "aws_s3_bucket.my_bucket",
			Before:   "private bucket",
			After:    "public bucket",
		},
	}

	return c.JSON(fiber.Map{
		"diff":   diff,
		"status": "ready",
	})
}