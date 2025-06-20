----------------
---- Common ----
----------------

CREATE TYPE icon AS (icon text, hex_color text);

----------------
---- Feeds -----
----------------

CREATE TYPE feed_status AS ENUM ('active', 'completed', 'suspended', 'broken');
CREATE TYPE feed_format AS ENUM ('rss', 'atom', 'json');
CREATE TABLE feed (
  id serial PRIMARY KEY,
  format feed_format NOT NULL,
  status feed_status NOT NULL DEFAULT 'active',
  link text NOT NULL UNIQUE,
  domain text, -- Optional because IP addresses aren't a domain

  title text NOT NULL,
  description text NOT NULL,
  icon text,
  language char(2), -- ISO 639-1 code

  skip_hours integer[24] NOT NULL DEFAULT '{}', -- 0 - 23
  skip_days_of_week integer[7] NOT NULL DEFAULT '{}', -- 0 = Sunday, 1 = Monday, ...
  ttl_in_minutes integer, -- Minimum time to cache the feed for
  etag text, -- ETag header from the last update

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW(), -- Time of the last update to the content
  fetched_at timestamptz NOT NULL DEFAULT NOW(), -- Time of the last fetch
  successful_fetch_at timestamptz NOT NULL DEFAULT NOW(), -- Time of the last successful fetch
  next_fetch_at timestamptz NOT NULL DEFAULT NOW() -- Time to fetch the feed next
);
CREATE INDEX feed_link ON feed (link);
CREATE INDEX feed_status ON feed (status);
CREATE INDEX feed_updated_at ON feed (updated_at);
CREATE INDEX feed_next_fetch_at ON feed (next_fetch_at);

----------------
-- Feed Items --
----------------

CREATE TYPE feed_item_enclosure AS (
  link text,
  mime text,
  length integer
);
CREATE TABLE feed_item (
  id bigserial PRIMARY KEY,
  feed_id integer NOT NULL REFERENCES feed (id) ON DELETE CASCADE,
  guid text NOT NULL, -- Globally unique identifier, defined by the feed
  index_in_feed integer NOT NULL, -- Index of the item in the feed, when it was queried

  title text NOT NULL,
  link text,
  description text,
  enclosure feed_item_enclosure,
  categories text[] NOT NULL DEFAULT '{}',
  comments_link text,
  published_at timestamptz,

  content text,
  content_type text,
  base_link text,

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
CREATE INDEX feed_item_feed_id ON feed_item (feed_id);
CREATE INDEX feed_item_link ON feed_item (link);
CREATE INDEX feed_item_index_in_feed ON feed_item (index_in_feed DESC);
CREATE INDEX feed_item_updated_at ON feed_item (updated_at DESC);

CREATE TABLE feed_item_parsed (
  id bigserial PRIMARY KEY,
  feed_item_id bigint NOT NULL REFERENCES feed_item (id) ON DELETE CASCADE,
  content text NOT NULL,
  content_type text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
CREATE INDEX feed_item_parsed_feed_item_id ON feed_item_parsed (feed_item_id);

----------------
----- User -----
----------------

CREATE TABLE "user" (
  id serial PRIMARY KEY,

  email text NOT NULL UNIQUE,
  username text NOT NULL UNIQUE, -- TODO: use as primary key?
  providers text[] NOT NULL DEFAULT '{}',
  password_hash text,
  passwordless_pub_key text[] NOT NULL DEFAULT '{}',

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);

CREATE TYPE user_oauth_client AS ENUM ('web', 'android', 'ios');
CREATE TABLE user_oauth_state (
  id serial PRIMARY KEY,
  client user_oauth_client NOT NULL,
  provider text NOT NULL,
  csrf_token text NOT NULL,
  pkce_verifier text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT NOW()
);
CREATE INDEX user_oauth_state_csrf_token ON user_oauth_state (csrf_token);
CREATE INDEX user_oauth_state_created_at ON user_oauth_state (created_at);

CREATE TABLE user_email_verification (
  id serial PRIMARY KEY,
  email text NOT NULL,
  code text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT NOW()
);

CREATE TABLE user_password_reset_codes (
  id serial PRIMARY KEY,
  email text NOT NULL,
  code text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT NOW()
);
CREATE INDEX user_email_verification_code ON user_email_verification (code);
CREATE INDEX user_email_verification_created_at ON user_email_verification (created_at);

----------------
---- Index -----
----------------

CREATE TABLE user_index (
  id serial PRIMARY KEY,
  owner integer NOT NULL REFERENCES "user" (id) ON DELETE CASCADE,

  query text NOT NULL,
  sort text NOT NULL,

  title text NOT NULL,
  description text,
  icon icon NOT NULL,

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
CREATE INDEX user_index_owner ON user_index (owner);

CREATE TYPE tag_child_type AS ENUM ('feed', 'index');
CREATE TYPE tag_child AS (type tag_child_type, id integer);
CREATE TABLE user_tag (
  id serial PRIMARY KEY,
  owner integer NOT NULL REFERENCES "user" (id) ON DELETE CASCADE,

  title text NOT NULL,
  children tag_child[] NOT NULL DEFAULT '{}',

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
CREATE INDEX user_tag_owner ON user_tag (owner);

CREATE TABLE user_feed (
  id serial PRIMARY KEY,
  owner integer NOT NULL REFERENCES "user" (id) ON DELETE CASCADE,
  feed integer NOT NULL REFERENCES feed (id) ON DELETE CASCADE,

  title text NOT NULL,
  icon icon,

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);

----------------
----- List -----
----------------

CREATE TABLE user_list (
  id serial PRIMARY KEY,
  owner integer NOT NULL REFERENCES "user" (id) ON DELETE CASCADE,
  title text NOT NULL,
  description text,
  icon icon,

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
CREATE INDEX user_list_owner ON user_list (owner);

CREATE TABLE user_list_item (
  id serial PRIMARY KEY,
  index integer NOT NULL,
  owner integer NOT NULL REFERENCES "user" (id) ON DELETE CASCADE,
  list integer NOT NULL REFERENCES user_list (id) ON DELETE CASCADE,
  item bigint NOT NULL REFERENCES feed_item (id) ON DELETE CASCADE,

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
CREATE INDEX user_list_item_index ON user_list_item (index);
CREATE INDEX user_list_item_owner ON user_list_item (owner);
CREATE INDEX user_list_item_list ON user_list_item (list);
CREATE INDEX user_list_item_item ON user_list_item (item);

CREATE TABLE user_history_item (
  id serial PRIMARY KEY,
  owner integer NOT NULL REFERENCES "user" (id) ON DELETE CASCADE,
  item bigint NOT NULL REFERENCES feed_item (id) ON DELETE CASCADE,
  progress double precision NOT NULL DEFAULT 0, -- Progress in the item, 0 - 1

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
CREATE INDEX user_history_item_owner ON user_history_item (owner);
CREATE INDEX user_history_item_item ON user_history_item (item);
CREATE INDEX user_history_item_progress ON user_history_item (progress);
CREATE INDEX user_history_item_updated_at ON user_history_item (updated_at);
