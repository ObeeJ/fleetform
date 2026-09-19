package store

import (
	"crypto/rand"
	"crypto/sha256"
	"database/sql"
	"encoding/hex"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	_ "modernc.org/sqlite"
)

type Store struct {
	DB *sql.DB
}

type User struct {
	ID           string `json:"id"`
	Email        string `json:"email"`
	Name         string `json:"name"`
	PasswordHash string `json:"-"`
	CreatedAt    string `json:"createdAt"`
}

type Org struct {
	ID        string `json:"id"`
	Name      string `json:"name"`
	Slug      string `json:"slug"`
	OwnerID   string `json:"ownerId"`
	Plan      string `json:"plan"`
	CreatedAt string `json:"createdAt"`
}

type Workspace struct {
	ID        string `json:"id"`
	OrgID     string `json:"orgId"`
	Name      string `json:"name"`
	Env       string `json:"env"`
	CreatedAt string `json:"createdAt"`
}

type Run struct {
	ID           string `json:"id"`
	WorkspaceID  string `json:"workspaceId"`
	Kind         string `json:"kind"`
	Status       string `json:"status"`
	Summary      string `json:"summary"`
	AddCount     int    `json:"addCount"`
	ChangeCount  int    `json:"changeCount"`
	DestroyCount int    `json:"destroyCount"`
	Idempotency  string `json:"idempotencyKey,omitempty"`
	CreatedAt    string `json:"createdAt"`
}

type APIKey struct {
	ID        string `json:"id"`
	OrgID     string `json:"orgId"`
	Name      string `json:"name"`
	Prefix    string `json:"prefix"`
	Last4     string `json:"last4"`
	CreatedAt string `json:"createdAt"`
}

func Open(path string) (*Store, error) {
	if path == "" {
		path = filepath.Join(".", "fleetform.db")
	}
	if dir := filepath.Dir(path); dir != "." && dir != "" {
		_ = os.MkdirAll(dir, 0o755)
	}
	db, err := sql.Open("sqlite", path+"?_pragma=busy_timeout(5000)&_pragma=foreign_keys(ON)&_pragma=journal_mode(WAL)")
	if err != nil {
		return nil, err
	}
	s := &Store{DB: db}
	if err := s.migrate(); err != nil {
		return nil, err
	}
	return s, nil
}

func (s *Store) migrate() error {
	_, err := s.DB.Exec(`
CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  password_hash TEXT NOT NULL,
  created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id)
);
CREATE TABLE IF NOT EXISTS orgs (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  slug TEXT NOT NULL UNIQUE,
  owner_id TEXT NOT NULL,
  plan TEXT NOT NULL DEFAULT 'free',
  created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS memberships (
  org_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  role TEXT NOT NULL,
  PRIMARY KEY (org_id, user_id)
);
CREATE TABLE IF NOT EXISTS workspaces (
  id TEXT PRIMARY KEY,
  org_id TEXT NOT NULL,
  name TEXT NOT NULL,
  env TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE(org_id, name)
);
CREATE TABLE IF NOT EXISTS runs (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  status TEXT NOT NULL,
  summary TEXT NOT NULL DEFAULT '',
  add_count INTEGER NOT NULL DEFAULT 0,
  change_count INTEGER NOT NULL DEFAULT 0,
  destroy_count INTEGER NOT NULL DEFAULT 0,
  idempotency TEXT,
  created_at TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS runs_idempotency ON runs(workspace_id, idempotency) WHERE idempotency IS NOT NULL AND idempotency != '';
CREATE TABLE IF NOT EXISTS api_keys (
  id TEXT PRIMARY KEY,
  org_id TEXT NOT NULL,
  name TEXT NOT NULL,
  prefix TEXT NOT NULL,
  last4 TEXT NOT NULL,
  hash TEXT NOT NULL,
  created_at TEXT NOT NULL
);
`)
	return err
}

func NewID(prefix string) string {
	var b [12]byte
	_, _ = rand.Read(b[:])
	return prefix + "_" + hex.EncodeToString(b[:])
}

func Now() string { return time.Now().UTC().Format(time.RFC3339) }

func Slugify(s string) string {
	s = strings.ToLower(strings.TrimSpace(s))
	var out []rune
	prevDash := false
	for _, r := range s {
		if (r >= 'a' && r <= 'z') || (r >= '0' && r <= '9') {
			out = append(out, r)
			prevDash = false
			continue
		}
		if !prevDash {
			out = append(out, '-')
			prevDash = true
		}
	}
	res := strings.Trim(string(out), "-")
	if res == "" {
		return "org"
	}
	return res
}

func HashToken(raw string) string {
	sum := sha256.Sum256([]byte(raw))
	return hex.EncodeToString(sum[:])
}

func (s *Store) CreateUser(email, name, hash string) (*User, error) {
	u := &User{ID: NewID("usr"), Email: strings.ToLower(email), Name: name, PasswordHash: hash, CreatedAt: Now()}
	_, err := s.DB.Exec(`INSERT INTO users(id,email,name,password_hash,created_at) VALUES(?,?,?,?,?)`,
		u.ID, u.Email, u.Name, u.PasswordHash, u.CreatedAt)
	return u, err
}

func (s *Store) UserByEmail(email string) (*User, error) {
	return s.scanUser(s.DB.QueryRow(`SELECT id,email,name,password_hash,created_at FROM users WHERE email=?`, strings.ToLower(email)))
}

func (s *Store) UserByID(id string) (*User, error) {
	return s.scanUser(s.DB.QueryRow(`SELECT id,email,name,password_hash,created_at FROM users WHERE id=?`, id))
}

func (s *Store) scanUser(row *sql.Row) (*User, error) {
	var u User
	if err := row.Scan(&u.ID, &u.Email, &u.Name, &u.PasswordHash, &u.CreatedAt); err != nil {
		return nil, err
	}
	return &u, nil
}

func (s *Store) CreateSession(userID string, ttl time.Duration) (string, time.Time, error) {
	id := NewID("ses")
	exp := time.Now().UTC().Add(ttl)
	_, err := s.DB.Exec(`INSERT INTO sessions(id,user_id,expires_at) VALUES(?,?,?)`, id, userID, exp.Format(time.RFC3339))
	return id, exp, err
}

func (s *Store) SessionUser(sessionID string) (*User, error) {
	var userID, exp string
	err := s.DB.QueryRow(`SELECT user_id, expires_at FROM sessions WHERE id=?`, sessionID).Scan(&userID, &exp)
	if err != nil {
		return nil, err
	}
	t, err := time.Parse(time.RFC3339, exp)
	if err != nil || t.Before(time.Now().UTC()) {
		return nil, sql.ErrNoRows
	}
	return s.UserByID(userID)
}

func (s *Store) DeleteSession(id string) error {
	_, err := s.DB.Exec(`DELETE FROM sessions WHERE id=?`, id)
	return err
}

func (s *Store) CreateOrg(owner *User, name, plan string) (*Org, error) {
	base := Slugify(name)
	slug := base
	for i := 0; i < 8; i++ {
		var n int
		_ = s.DB.QueryRow(`SELECT COUNT(*) FROM orgs WHERE slug=?`, slug).Scan(&n)
		if n == 0 {
			break
		}
		slug = fmt.Sprintf("%s-%d", base, i+2)
	}
	o := &Org{ID: NewID("org"), Name: name, Slug: slug, OwnerID: owner.ID, Plan: plan, CreatedAt: Now()}
	tx, err := s.DB.Begin()
	if err != nil {
		return nil, err
	}
	if _, err = tx.Exec(`INSERT INTO orgs(id,name,slug,owner_id,plan,created_at) VALUES(?,?,?,?,?,?)`,
		o.ID, o.Name, o.Slug, o.OwnerID, o.Plan, o.CreatedAt); err != nil {
		_ = tx.Rollback()
		return nil, err
	}
	if _, err = tx.Exec(`INSERT INTO memberships(org_id,user_id,role) VALUES(?,?,?)`, o.ID, owner.ID, "owner"); err != nil {
		_ = tx.Rollback()
		return nil, err
	}
	if err = tx.Commit(); err != nil {
		return nil, err
	}
	return o, nil
}

func (s *Store) OrgsForUser(userID string) ([]Org, error) {
	rows, err := s.DB.Query(`
SELECT o.id,o.name,o.slug,o.owner_id,o.plan,o.created_at
FROM orgs o JOIN memberships m ON m.org_id=o.id
WHERE m.user_id=? ORDER BY o.created_at DESC`, userID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []Org
	for rows.Next() {
		var o Org
		if err := rows.Scan(&o.ID, &o.Name, &o.Slug, &o.OwnerID, &o.Plan, &o.CreatedAt); err != nil {
			return nil, err
		}
		out = append(out, o)
	}
	if out == nil {
		out = []Org{}
	}
	return out, rows.Err()
}

func (s *Store) Membership(orgID, userID string) (string, error) {
	var role string
	err := s.DB.QueryRow(`SELECT role FROM memberships WHERE org_id=? AND user_id=?`, orgID, userID).Scan(&role)
	return role, err
}

func (s *Store) OrgByID(id string) (*Org, error) {
	var o Org
	err := s.DB.QueryRow(`SELECT id,name,slug,owner_id,plan,created_at FROM orgs WHERE id=?`, id).Scan(
		&o.ID, &o.Name, &o.Slug, &o.OwnerID, &o.Plan, &o.CreatedAt)
	if err != nil {
		return nil, err
	}
	return &o, nil
}

func (s *Store) SetPlan(orgID, plan string) error {
	_, err := s.DB.Exec(`UPDATE orgs SET plan=? WHERE id=?`, plan, orgID)
	return err
}

func (s *Store) CreateWorkspace(orgID, name, env string) (*Workspace, error) {
	w := &Workspace{ID: NewID("ws"), OrgID: orgID, Name: name, Env: env, CreatedAt: Now()}
	_, err := s.DB.Exec(`INSERT INTO workspaces(id,org_id,name,env,created_at) VALUES(?,?,?,?,?)`,
		w.ID, w.OrgID, w.Name, w.Env, w.CreatedAt)
	return w, err
}

func (s *Store) Workspaces(orgID string) ([]Workspace, error) {
	rows, err := s.DB.Query(`SELECT id,org_id,name,env,created_at FROM workspaces WHERE org_id=? ORDER BY created_at DESC`, orgID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []Workspace
	for rows.Next() {
		var w Workspace
		if err := rows.Scan(&w.ID, &w.OrgID, &w.Name, &w.Env, &w.CreatedAt); err != nil {
			return nil, err
		}
		out = append(out, w)
	}
	if out == nil {
		out = []Workspace{}
	}
	return out, rows.Err()
}

func (s *Store) WorkspaceByID(id string) (*Workspace, error) {
	var w Workspace
	err := s.DB.QueryRow(`SELECT id,org_id,name,env,created_at FROM workspaces WHERE id=?`, id).Scan(
		&w.ID, &w.OrgID, &w.Name, &w.Env, &w.CreatedAt)
	if err != nil {
		return nil, err
	}
	return &w, nil
}

func (s *Store) CreateRun(wsID, kind, idem string) (*Run, error) {
	if idem != "" {
		var existing Run
		err := s.DB.QueryRow(`SELECT id,workspace_id,kind,status,summary,add_count,change_count,destroy_count,IFNULL(idempotency,''),created_at FROM runs WHERE workspace_id=? AND idempotency=?`, wsID, idem).
			Scan(&existing.ID, &existing.WorkspaceID, &existing.Kind, &existing.Status, &existing.Summary, &existing.AddCount, &existing.ChangeCount, &existing.DestroyCount, &existing.Idempotency, &existing.CreatedAt)
		if err == nil {
			return &existing, nil
		}
	}
	r := &Run{
		ID: NewID("run"), WorkspaceID: wsID, Kind: kind, Status: "planned",
		Summary: "Plan computed from workspace configuration.",
		AddCount: 1, ChangeCount: 0, DestroyCount: 0,
		Idempotency: idem, CreatedAt: Now(),
	}
	if kind == "apply" {
		r.Status = "applied"
		r.Summary = "Apply recorded. Provider execution is still a controlled preview in this release."
	}
	if kind == "destroy" {
		r.Status = "planned"
		r.DestroyCount = 1
		r.AddCount = 0
		r.Summary = "Destroy plan recorded. No live resources were mutated."
	}
	_, err := s.DB.Exec(`INSERT INTO runs(id,workspace_id,kind,status,summary,add_count,change_count,destroy_count,idempotency,created_at) VALUES(?,?,?,?,?,?,?,?,?,?)`,
		r.ID, r.WorkspaceID, r.Kind, r.Status, r.Summary, r.AddCount, r.ChangeCount, r.DestroyCount, nullIfEmpty(idem), r.CreatedAt)
	return r, err
}

func nullIfEmpty(s string) any {
	if s == "" {
		return nil
	}
	return s
}

func (s *Store) RunsForOrg(orgID string) ([]Run, error) {
	rows, err := s.DB.Query(`
SELECT r.id,r.workspace_id,r.kind,r.status,r.summary,r.add_count,r.change_count,r.destroy_count,IFNULL(r.idempotency,''),r.created_at
FROM runs r JOIN workspaces w ON w.id=r.workspace_id
WHERE w.org_id=? ORDER BY r.created_at DESC LIMIT 50`, orgID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []Run
	for rows.Next() {
		var r Run
		if err := rows.Scan(&r.ID, &r.WorkspaceID, &r.Kind, &r.Status, &r.Summary, &r.AddCount, &r.ChangeCount, &r.DestroyCount, &r.Idempotency, &r.CreatedAt); err != nil {
			return nil, err
		}
		out = append(out, r)
	}
	if out == nil {
		out = []Run{}
	}
	return out, rows.Err()
}

func (s *Store) CreateAPIKey(orgID, name string) (raw string, key *APIKey, err error) {
	var extra [8]byte
	_, _ = rand.Read(extra[:])
	raw = "ff_" + hex.EncodeToString(extra[:]) + hex.EncodeToString(extra[:])
	key = &APIKey{
		ID: NewID("key"), OrgID: orgID, Name: name,
		Prefix: raw[:7], Last4: raw[len(raw)-4:], CreatedAt: Now(),
	}
	_, err = s.DB.Exec(`INSERT INTO api_keys(id,org_id,name,prefix,last4,hash,created_at) VALUES(?,?,?,?,?,?,?)`,
		key.ID, key.OrgID, key.Name, key.Prefix, key.Last4, HashToken(raw), key.CreatedAt)
	return raw, key, err
}

func (s *Store) APIKeys(orgID string) ([]APIKey, error) {
	rows, err := s.DB.Query(`SELECT id,org_id,name,prefix,last4,created_at FROM api_keys WHERE org_id=? ORDER BY created_at DESC`, orgID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []APIKey
	for rows.Next() {
		var k APIKey
		if err := rows.Scan(&k.ID, &k.OrgID, &k.Name, &k.Prefix, &k.Last4, &k.CreatedAt); err != nil {
			return nil, err
		}
		out = append(out, k)
	}
	if out == nil {
		out = []APIKey{}
	}
	return out, rows.Err()
}

func (s *Store) DeleteAPIKey(orgID, keyID string) error {
	res, err := s.DB.Exec(`DELETE FROM api_keys WHERE id=? AND org_id=?`, keyID, orgID)
	if err != nil {
		return err
	}
	n, _ := res.RowsAffected()
	if n == 0 {
		return sql.ErrNoRows
	}
	return nil
}
