package handlers

import (
	"encoding/json"
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/websocket/v2"
	"io/ioutil"
	"time"
)

func RealtimeHandler(conn *websocket.Conn) {
	for {
		data, err := ioutil.ReadFile("../fleetform_plan.json")
		if err != nil {
			conn.WriteJSON(fiber.Map{"error": "Failed to read plan"})
			return
		}
		var planData map[string]interface{}
		json.Unmarshal(data, &planData)
		conn.WriteJSON(fiber.Map{"event": "plan_update", "data": planData})
		time.Sleep(time.Second * 2)
	}
}