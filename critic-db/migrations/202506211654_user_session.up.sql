-- The table holding user sessions
CREATE TABLE user_session (
	id INT NOT NULL,
	username TEXT NOT NULL UNIQUE,
	access_token TEXT NOT NULL,
	refresh_token TEXT NOT NULL,
	expires_at TIMESTAMPTZ NOT NULL
);
