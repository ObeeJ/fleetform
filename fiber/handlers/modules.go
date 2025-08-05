package handlers

import (
	"github.com/gofiber/fiber/v2"
	"io/ioutil"
)

func ModulesHandler(c *fiber.Ctx) error {
	files, err := ioutil.ReadDir("../modules")
	if err != nil {
		return c.Status(500).JSON(fiber.Map{"error": "Failed to read modules directory"})
	}
	
	var modules []string
	for _, file := range files {
		if file.IsDir() {
			modules = append(modules, file.Name())
		}
	}
	
	return c.JSON(fiber.Map{
		"modules": modules,
		"count":   len(modules),
	})
}