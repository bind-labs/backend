# Backend

## Getting Started

1. Run a local Postgres instance: `docker run -p 5432:5432 -e POSTGRES_PASSWORD=bind -e POSTGRES_USER=bind -e POSTGRES_DB=bind --name bind-postgres -d postgres`
2. Copy the `.env.example` file to `.env`
3. Enter the nix shell: `nix develop`
4. Run `cargo watch -x run`
5. Run `psql -c "DELETE FROM _sqlx_migrations"` when you need to delete migrations

## Endpoints

Prefixed under `/api/v1`

- Feeds `/feed`
  - `GET/PUT /` Listing or creating feeds, not the content
  - `GET /:id`

- Index `/index`
  - ID can be derived from the parameters and then hashed for caching
  - Two types: One shows the content as-is (simple interleave for now), the other builds it from the RSS history
  - `GET /:id` Gets the actual content of the feed
    - `GET /me` for home feed
  - `GET/PUT /` Lists or creates an index
  - `GET/UPDATE/DELETE /:id`

- Articles, Podcasts, Youtube, Posts `/item`
  - ID derived from the URL, hashed
  - `GET /:id`
    - `GET /parse` extracts content from the url
    - `GET /comments` gets the comments
  - List `/list`
    - `GET/PUT /` Lists the lists or creates a new one
    - `UPDATE/DELETE /:id`
    - `/:id`
      - `GET /` List items
      - `PUT /` Create a list
      - `PUT/UPDATE/DELETE /:item_id` Creates/updates/deletes an item in the list

- Tags `/tags`

- User `/user`
  - `/:id`
    - `GET/UPDATE /settings`
  - Token `/token`
    - `POST /refresh` Refreshes the token
    - `GET /status` Checks if the user is logged in and returns the user info
  - Email `/email`
    - `POST /verify` Sends a verification email
    - `POST /login`
    - `POST /register` Registers the user
    - `POST /reset-password` TODO: magic link instead?
  - OAuth `/oauth`
    - `GET /providers` List available providers
    - `GET /authorize?provider=id` Redirects to a provider's authorization page
    - `GET /callback` Exchanges the code for an access token and returns the user
  - History `/history`
    - `GET/PUT /` Lists or adds to history
    - `UPDATE/DELETE /:id`

- Search `/search`
  - `GET /:query` How does this work? Postgres full text search. Use feedbin for inspiration

## Roadmap

- [x] API Schema design
- [x] Axum template with SQLx
- [x] Postgres schema
- [ ] RSS parsing and caching
  - [x] Support `Retry-After` header on 429
  - [x] Use `If-Modified-Since` header to get 304
  - [x] Use `ETag` header to get 304
  - [x] Support `max-age`
  - [ ] Support `<sy:updatePeriod>` and `<sy:updateFrequency>`
  - [x] Follow redirect and update link to the feed
  - [ ] JSON Feed support using Serde
  - [ ] Feed discovery
    - [ ] Search via Kagi Search API
    - [ ] Search again with `site:domain.com rss`
      - Take any links that include feed/rss/atom at the end of the path
    - [ ] Search for youtube channels
    - [ ] Search existing feeds
- [ ] Website parsing
- [x] API Implementation
  - [x] Feeds
  - [x] Index
  - [ ] Items
    - [ ] Lists
  - [x] Users
    - [x] Authentication
  - [ ] Search
