package httpapi

import (
	"database/sql"
	"os"
	"strings"
	"time"

	"github.com/ObeeJ/fleetform/fiber/internal/store"
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/websocket/v2"
	"golang.org/x/crypto/bcrypt"
)

const cookieName = "fleetform_session"

type API struct{ Store *store.Store }

func New(s *store.Store) *API { return &API{Store: s} }

func (a *API) Register(app *fiber.App) {
	api := app.Group("/api")
	api.Post("/auth/register", a.register)
	api.Post("/auth/login", a.login)
	api.Post("/auth/logout", a.logout)
	api.Get("/auth/me", a.me)
	api.Get("/orgs", a.requireUser, a.listOrgs)
	api.Post("/orgs", a.requireUser, a.createOrg)
	api.Post("/orgs/:orgId/plan", a.requireUser, a.setPlan)
	api.Get("/orgs/:orgId/workspaces", a.requireUser, a.listWorkspaces)
	api.Post("/orgs/:orgId/workspaces", a.requireUser, a.createWorkspace)
	api.Get("/orgs/:orgId/runs", a.requireUser, a.listRuns)
	api.Post("/orgs/:orgId/workspaces/:wsId/runs", a.requireUser, a.createRun)
	api.Get("/orgs/:orgId/keys", a.requireUser, a.listKeys)
	api.Post("/orgs/:orgId/keys", a.requireUser, a.createKey)
	api.Delete("/orgs/:orgId/keys/:keyId", a.requireUser, a.deleteKey)
	api.Get("/orgs/:orgId/overview", a.requireUser, a.overview)
	app.Get("/realtime", websocket.New(func(c *websocket.Conn) {
		_ = c.WriteJSON(fiber.Map{"status": "connected", "service": "fleetform"})
		for {
			if _, _, err := c.ReadMessage(); err != nil {
				break
			}
		}
	}))
}

func fail(c *fiber.Ctx, status int, code, msg string) error {
	return c.Status(status).JSON(fiber.Map{"error": code, "message": msg})
}
func userOf(c *fiber.Ctx) *store.User { return c.Locals("user").(*store.User) }
func publicUser(u *store.User) fiber.Map {
	return fiber.Map{"id": u.ID, "email": u.Email, "name": u.Name, "createdAt": u.CreatedAt}
}
func (a *API) currentUser(c *fiber.Ctx) (*store.User, error) {
	sid := c.Cookies(cookieName)
	if sid == "" {
		return nil, sql.ErrNoRows
	}
	return a.Store.SessionUser(sid)
}
func (a *API) requireUser(c *fiber.Ctx) error {
	u, err := a.currentUser(c)
	if err != nil {
		return fail(c, 401, "not_authenticated", "Sign in required.")
	}
	c.Locals("user", u)
	return c.Next()
}
func (a *API) requireOrgRole(c *fiber.Ctx, orgID string) (string, error) {
	return a.Store.Membership(orgID, userOf(c).ID)
}
func setSession(c *fiber.Ctx, id string, exp time.Time) {
	secure := os.Getenv("FLEETFORM_SECURE_COOKIE") == "1" || c.Protocol() == "https"
	c.Cookie(&fiber.Cookie{Name: cookieName, Value: id, Expires: exp, HTTPOnly: true, Secure: secure, SameSite: "Lax", Path: "/"})
}

type authBody struct {
	Email    string `json:"email"`
	Password string `json:"password"`
	Name     string `json:"name"`
	OrgName  string `json:"orgName"`
}
type nameBody struct {
	Name string `json:"name"`
	Env  string `json:"env"`
	Plan string `json:"plan"`
	Kind string `json:"kind"`
}

func (a *API) register(c *fiber.Ctx) error {
	var body authBody
	if err := c.BodyParser(&body); err != nil {
		return fail(c, 400, "invalid_json", "Request body must be JSON.")
	}
	body.Email = strings.TrimSpace(strings.ToLower(body.Email))
	body.Name = strings.TrimSpace(body.Name)
	body.OrgName = strings.TrimSpace(body.OrgName)
	if body.Email == "" || !strings.Contains(body.Email, "@") {
		return fail(c, 400, "invalid_email", "A valid email is required.")
	}
	if len(body.Password) < 10 {
		return fail(c, 400, "weak_password", "Password must be at least 10 characters.")
	}
	if body.Name == "" {
		body.Name = strings.Split(body.Email, "@")[0]
	}
	if body.OrgName == "" {
		body.OrgName = body.Name + "'s workspace"
	}
	if _, err := a.Store.UserByEmail(body.Email); err == nil {
		return fail(c, 409, "email_taken", "An account with that email already exists.")
	}
	hash, err := bcrypt.GenerateFromPassword([]byte(body.Password), 12)
	if err != nil {
		return fail(c, 500, "hash_failed", "Could not create account.")
	}
	u, err := a.Store.CreateUser(body.Email, body.Name, string(hash))
	if err != nil {
		return fail(c, 500, "create_failed", "Could not create account.")
	}
	org, err := a.Store.CreateOrg(u, body.OrgName, "free")
	if err != nil {
		return fail(c, 500, "org_failed", "Account created but organization setup failed.")
	}
	if _, err := a.Store.CreateWorkspace(org.ID, "production", "production"); err != nil {
		return fail(c, 500, "workspace_failed", "Default workspace could not be created.")
	}
	sid, exp, err := a.Store.CreateSession(u.ID, 7*24*time.Hour)
	if err != nil {
		return fail(c, 500, "session_failed", "Account created. Sign in to continue.")
	}
	setSession(c, sid, exp)
	return c.Status(201).JSON(fiber.Map{"user": publicUser(u), "org": org})
}

func (a *API) login(c *fiber.Ctx) error {
	var body authBody
	if err := c.BodyParser(&body); err != nil {
		return fail(c, 400, "invalid_json", "Request body must be JSON.")
	}
	u, err := a.Store.UserByEmail(strings.TrimSpace(body.Email))
	if err != nil {
		return fail(c, 401, "invalid_credentials", "Email or password is incorrect.")
	}
	if bcrypt.CompareHashAndPassword([]byte(u.PasswordHash), []byte(body.Password)) != nil {
		return fail(c, 401, "invalid_credentials", "Email or password is incorrect.")
	}
	sid, exp, err := a.Store.CreateSession(u.ID, 7*24*time.Hour)
	if err != nil {
		return fail(c, 500, "session_failed", "Could not start a session.")
	}
	setSession(c, sid, exp)
	return c.JSON(fiber.Map{"user": publicUser(u)})
}

func (a *API) logout(c *fiber.Ctx) error {
	sid := c.Cookies(cookieName)
	if sid != "" {
		_ = a.Store.DeleteSession(sid)
	}
	c.Cookie(&fiber.Cookie{Name: cookieName, Value: "", Expires: time.Unix(0, 0), HTTPOnly: true, Path: "/"})
	return c.JSON(fiber.Map{"ok": true})
}

func (a *API) me(c *fiber.Ctx) error {
	u, err := a.currentUser(c)
	if err != nil {
		return fail(c, 401, "not_authenticated", "Sign in required.")
	}
	orgs, err := a.Store.OrgsForUser(u.ID)
	if err != nil {
		return fail(c, 500, "orgs_failed", "Could not load organizations.")
	}
	return c.JSON(fiber.Map{"user": publicUser(u), "orgs": orgs})
}

func (a *API) listOrgs(c *fiber.Ctx) error {
	orgs, err := a.Store.OrgsForUser(userOf(c).ID)
	if err != nil {
		return fail(c, 500, "list_failed", "Could not list organizations.")
	}
	return c.JSON(fiber.Map{"orgs": orgs})
}
func (a *API) createOrg(c *fiber.Ctx) error {
	var body nameBody
	if err := c.BodyParser(&body); err != nil || strings.TrimSpace(body.Name) == "" {
		return fail(c, 400, "invalid_name", "Organization name is required.")
	}
	org, err := a.Store.CreateOrg(userOf(c), strings.TrimSpace(body.Name), "free")
	if err != nil {
		return fail(c, 500, "create_failed", "Could not create organization.")
	}
	return c.Status(201).JSON(org)
}
func (a *API) setPlan(c *fiber.Ctx) error {
	orgID := c.Params("orgId")
	role, err := a.requireOrgRole(c, orgID)
	if err != nil {
		return fail(c, 404, "not_found", "Organization not found.")
	}
	if role != "owner" {
		return fail(c, 403, "forbidden", "Only owners can change the plan.")
	}
	var body nameBody
	if err := c.BodyParser(&body); err != nil {
		return fail(c, 400, "invalid_json", "Request body must be JSON.")
	}
	allowed := map[string]bool{"free": true, "lite": true, "plus": true, "pro": true, "ultra": true}
	if !allowed[body.Plan] {
		return fail(c, 400, "invalid_plan", "Unknown plan.")
	}
	if err := a.Store.SetPlan(orgID, body.Plan); err != nil {
		return fail(c, 500, "update_failed", "Could not update plan.")
	}
	org, _ := a.Store.OrgByID(orgID)
	return c.JSON(org)
}
func (a *API) listWorkspaces(c *fiber.Ctx) error {
	orgID := c.Params("orgId")
	if _, err := a.requireOrgRole(c, orgID); err != nil {
		return fail(c, 404, "not_found", "Organization not found.")
	}
	list, err := a.Store.Workspaces(orgID)
	if err != nil {
		return fail(c, 500, "list_failed", "Could not list workspaces.")
	}
	return c.JSON(fiber.Map{"workspaces": list})
}
func (a *API) createWorkspace(c *fiber.Ctx) error {
	orgID := c.Params("orgId")
	if _, err := a.requireOrgRole(c, orgID); err != nil {
		return fail(c, 404, "not_found", "Organization not found.")
	}
	var body nameBody
	if err := c.BodyParser(&body); err != nil || strings.TrimSpace(body.Name) == "" {
		return fail(c, 400, "invalid_name", "Workspace name is required.")
	}
	env := body.Env
	if env == "" {
		env = "development"
	}
	w, err := a.Store.CreateWorkspace(orgID, strings.TrimSpace(body.Name), env)
	if err != nil {
		return fail(c, 500, "create_failed", "Could not create workspace.")
	}
	return c.Status(201).JSON(w)
}
func (a *API) listRuns(c *fiber.Ctx) error {
	orgID := c.Params("orgId")
	if _, err := a.requireOrgRole(c, orgID); err != nil {
		return fail(c, 404, "not_found", "Organization not found.")
	}
	list, err := a.Store.RunsForOrg(orgID)
	if err != nil {
		return fail(c, 500, "list_failed", "Could not list runs.")
	}
	return c.JSON(fiber.Map{"runs": list})
}
func (a *API) createRun(c *fiber.Ctx) error {
	orgID := c.Params("orgId")
	wsID := c.Params("wsId")
	if _, err := a.requireOrgRole(c, orgID); err != nil {
		return fail(c, 404, "not_found", "Organization not found.")
	}
	ws, err := a.Store.WorkspaceByID(wsID)
	if err != nil || ws.OrgID != orgID {
		return fail(c, 404, "not_found", "Workspace not found.")
	}
	var body nameBody
	_ = c.BodyParser(&body)
	kind := body.Kind
	if kind == "" {
		kind = "plan"
	}
	if kind != "plan" && kind != "apply" && kind != "destroy" {
		return fail(c, 400, "invalid_kind", "kind must be plan, apply, or destroy.")
	}
	run, err := a.Store.CreateRun(wsID, kind, strings.TrimSpace(c.Get("Idempotency-Key")))
	if err != nil {
		return fail(c, 500, "create_failed", "Could not create run.")
	}
	return c.Status(201).JSON(run)
}
func (a *API) listKeys(c *fiber.Ctx) error {
	orgID := c.Params("orgId")
	if _, err := a.requireOrgRole(c, orgID); err != nil {
		return fail(c, 404, "not_found", "Organization not found.")
	}
	list, err := a.Store.APIKeys(orgID)
	if err != nil {
		return fail(c, 500, "list_failed", "Could not list keys.")
	}
	return c.JSON(fiber.Map{"keys": list})
}
func (a *API) createKey(c *fiber.Ctx) error {
	orgID := c.Params("orgId")
	role, err := a.requireOrgRole(c, orgID)
	if err != nil {
		return fail(c, 404, "not_found", "Organization not found.")
	}
	if role != "owner" && role != "admin" {
		return fail(c, 403, "forbidden", "Only owners and admins can create API keys.")
	}
	var body nameBody
	_ = c.BodyParser(&body)
	name := strings.TrimSpace(body.Name)
	if name == "" {
		name = "default"
	}
	raw, key, err := a.Store.CreateAPIKey(orgID, name)
	if err != nil {
		return fail(c, 500, "create_failed", "Could not create API key.")
	}
	return c.Status(201).JSON(fiber.Map{"key": key, "secret": raw, "warning": "Copy this secret now. It is not stored in plaintext."})
}
func (a *API) deleteKey(c *fiber.Ctx) error {
	orgID := c.Params("orgId")
	role, err := a.requireOrgRole(c, orgID)
	if err != nil {
		return fail(c, 404, "not_found", "Organization not found.")
	}
	if role != "owner" && role != "admin" {
		return fail(c, 403, "forbidden", "Only owners and admins can revoke API keys.")
	}
	if err := a.Store.DeleteAPIKey(orgID, c.Params("keyId")); err != nil {
		return fail(c, 404, "not_found", "API key not found.")
	}
	return c.JSON(fiber.Map{"ok": true})
}
func (a *API) overview(c *fiber.Ctx) error {
	orgID := c.Params("orgId")
	if _, err := a.requireOrgRole(c, orgID); err != nil {
		return fail(c, 404, "not_found", "Organization not found.")
	}
	org, err := a.Store.OrgByID(orgID)
	if err != nil {
		return fail(c, 404, "not_found", "Organization not found.")
	}
	ws, _ := a.Store.Workspaces(orgID)
	runs, _ := a.Store.RunsForOrg(orgID)
	keys, _ := a.Store.APIKeys(orgID)
	return c.JSON(fiber.Map{"org": org, "workspaces": ws, "runs": runs, "keys": keys, "limits": planLimits(org.Plan), "claimedNote": "Plan and apply runs are recorded and idempotent. Live cloud mutation remains gated until the Rust provisioner is wired to this control plane."})
}
func planLimits(plan string) fiber.Map {
	switch plan {
	case "lite":
		return fiber.Map{"workspaces": 10, "team": 1, "runsPerMonth": 200, "priceUSD": 5}
	case "plus":
		return fiber.Map{"workspaces": 50, "team": 5, "runsPerMonth": 2000, "priceUSD": 15}
	case "pro":
		return fiber.Map{"workspaces": 200, "team": 20, "runsPerMonth": 20000, "priceUSD": 120}
	case "ultra":
		return fiber.Map{"workspaces": -1, "team": 100, "runsPerMonth": -1, "priceUSD": 400}
	default:
		return fiber.Map{"workspaces": 3, "team": 1, "runsPerMonth": 50, "priceUSD": 0}
	}
}
