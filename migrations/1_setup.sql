----------------
---- Common ----
----------------

CREATE TYPE icon AS (icon text, hex_color text);

----------------
---- Feeds -----
----------------

CREATE TABLE feed (
  id SERIAL PRIMARY KEY,
  link text NOT NULL UNIQUE,
  domain text NOT NULL,

  title text NOT NULL,
  description text,
  icon text,
  language char(2) NOT NULL DEFAULT 'en', -- ISO 639-1 code

  skip_hours integer[24] NOT NULL DEFAULT '{}', -- 0 - 23
  skip_days_of_week integer[7] NOT NULL DEFAULT '{}', -- 0 = Sunday, 1 = Monday, ...
  ttl_in_minutes integer NOT NULL DEFAULT 0, -- How long to cache the feed for
  suspended boolean NOT NULL DEFAULT false, -- Whether the feed should be updated

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
)
CREATE INDEX feed_link ON feed (link);
CREATE INDEX feed_last_updated ON feed (last_updated);
CREATE INDEX feed_suspended ON feed (suspended);

----------------
-- Feed Items --
----------------

CREATE TYPE feed_item_enclosure AS (
  link text,
  mime text,
  length integer
);
CREATE TABLE feed_item (
  id SERIAL PRIMARY KEY,
  feed_id integer NOT NULL REFERENCES feed (id),
  guid text, -- Globally unique identifier, defined by the feed
  index_in_feed integer NOT NULL UNIQUE, -- Index of the item in the feed, where larger numbers are more recent

  title text NOT NULL,
  link text,
  description text,
  enclosure feed_item_enclosure,
  content text,
  categories text[] NOT NULL DEFAULT '{}',
  comments_link text,
  published_at timestamptz,

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW(),
);
ALTER TABLE feed_item ADD CONSTRAINT feed_item_feed_id_fkey FOREIGN KEY (feed_id) REFERENCES feed (id) ON DELETE CASCADE;
CREATE INDEX feed_item_feed_id ON feed_item (feed_id);
CREATE INDEX feed_item_link ON feed_item (link);
CREATE INDEX feed_item_index_in_feed ON feed_item (index_in_feed DESC);

CREATE TABLE feed_item_parsed (
  id SERIAL PRIMARY KEY,
  feed_item_id integer NOT NULL REFERENCES feed_item (id),
  content text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
ALTER TABLE feed_item_parsed ADD CONSTRAINT feed_item_parsed_feed_item_id_fkey FOREIGN KEY (feed_item_id) REFERENCES feed_item (id) ON DELETE CASCADE;

----------------
---- Index -----
----------------

CREATE TABLE user_index (
  id SERIAL PRIMARY KEY,
  owner integer NOT NULL REFERENCES user (id),

  query text NOT NULL,
  sort text NOT NULL,

  title text NOT NULL,
  description text,
  icon icon NOT NULL,

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
ALTER TABLE user_index ADD CONSTRAINT user_index_owner_fkey FOREIGN KEY (owner) REFERENCES user (id) ON DELETE CASCADE;
CREATE INDEX user_index_owner ON user_index (owner);

CREATE TABLE user_tag (
  id SERIAL PRIMARY KEY,
  owner integer NOT NULL REFERENCES user (id),

  title text NOT NULL,
  children tag_child[] NOT NULL DEFAULT '{}',

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
ALTER TABLE user_tag ADD CONSTRAINT user_tag_owner_fkey FOREIGN KEY (owner) REFERENCES user (id) ON DELETE CASCADE;
CREATE TYPE tag_child AS (type tag_child_type, id integer);
CREATE TYPE tag_child_type AS ENUM ('feed', 'index');
CREATE INDEX user_tag_owner ON user_tag (owner);

CREATE TABLE user_feed (
  id SERIAL PRIMARY KEY,
  owner integer NOT NULL REFERENCES user (id),
  feed integer NOT NULL REFERENCES feed (id),

  title text NOT NULL,
  icon icon,

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
ALTER TABLE user_feed ADD CONSTRAINT user_feed_owner_fkey FOREIGN KEY (owner) REFERENCES user (id) ON DELETE CASCADE;
ALTER TABLE user_feed ADD CONSTRAINT user_feed_feed_fkey FOREIGN KEY (feed) REFERENCES feed (id) ON DELETE CASCADE;

----------------
----- List -----
----------------

CREATE TABLE user_list (
  id SERIAL PRIMARY KEY,
  owner integer NOT NULL REFERENCES user (id),
  title text NOT NULL,
  description text,
  icon icon,

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
ALTER TABLE user_list ADD CONSTRAINT user_list_owner_fkey FOREIGN KEY (owner) REFERENCES user (id) ON DELETE CASCADE;
CREATE INDEX user_list_owner ON user_list (owner);

CREATE TABLE user_list_item (
  id SERIAL PRIMARY KEY,
  index integer NOT NULL,
  owner integer NOT NULL REFERENCES user (id),
  list integer NOT NULL REFERENCES user_list (id),
  item integer NOT NULL REFERENCES feed_item (id),

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
ALTER TABLE user_list_item ADD CONSTRAINT user_list_item_owner_fkey FOREIGN KEY (owner) REFERENCES user (id) ON DELETE CASCADE;
ALTER TABLE user_list_item ADD CONSTRAINT user_list_item_list_fkey FOREIGN KEY (list) REFERENCES user_list (id) ON DELETE CASCADE;
ALTER TABLE user_list_item ADD CONSTRAINT user_list_item_item_fkey FOREIGN KEY (item) REFERENCES feed_item (id) ON DELETE CASCADE;
CREATE INDEX user_list_item_index ON user_list_item (index);
CREATE INDEX user_list_item_owner ON user_list_item (owner);
CREATE INDEX user_list_item_list ON user_list_item (list);
CREATE INDEX user_list_item_item ON user_list_item (item);

CREATE TABLE user_history_item (
  id SERIAL PRIMARY KEY,
  owner integer NOT NULL REFERENCES user (id),
  item integer NOT NULL REFERENCES feed_item (id),
  progress integer NOT NULL DEFAULT 0, -- Progress in the item, 0 - 100

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
ALTER TABLE user_history_item ADD CONSTRAINT user_history_item_owner_fkey FOREIGN KEY (owner) REFERENCES user (id) ON DELETE CASCADE;
ALTER TABLE user_history_item ADD CONSTRAINT user_history_item_item_fkey FOREIGN KEY (item) REFERENCES feed_item (id) ON DELETE CASCADE;
CREATE INDEX user_history_item_owner ON user_history_item (owner);
CREATE INDEX user_history_item_item ON user_history_item (item);
CREATE INDEX user_history_item_progress ON user_history_item (progress);
CREATE INDEX user_history_item_updated_at ON user_history_item (updated_at);

----------------
----- User -----
----------------

CREATE TABLE user (
  id SERIAL PRIMARY KEY,

  email text NOT NULL UNIQUE,
  username text NOT NULL UNIQUE,
  providers provider[] NOT NULL DEFAULT '{}',
  password_hash text,
  passwordless_pub_key text,
  refresh_tokens text[],

  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
CREATE TYPE provider AS ENUM ('google', 'github', 'apple');
