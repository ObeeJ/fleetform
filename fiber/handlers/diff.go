package handlers

import (
	"encoding/json"

	"github.com/gofiber/fiber/v2"
)

type DiffEntry struct {
	Action   string `json:"action"`
	Resource string `json:"resource"`
	Note     string `json:"note,omitempty"`
}

func DiffHandler(c *fiber.Ctx) error {
	data, _, err := readFirst("../fleetform_plan.json", "fleetform_plan.json")
	if err != nil {
		return c.JSON(fiber.Map{"diff": []DiffEntry{}, "status": "no-plan"})
	}

	var plan struct {
		Changes []struct {
			Address string `json:"address"`
			Action  string `json:"action"`
			Note    string `json:"note"`
		} `json:"changes"`
	}
	_ = json.Unmarshal(data, &plan)

	diff := make([]DiffEntry, 0, len(plan.Changes))
	for _, ch := range plan.Changes {
		diff = append(diff, DiffEntry{Action: ch.Action, Resource: ch.Address, Note: ch.Note})
	}
	return c.JSON(fiber.Map{"diff": diff, "status": "ready"})
}
