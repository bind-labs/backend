# Backend

## Endpoints

Prefixed under `/api/v1`

- Feeds `/feed`
  - `GET/PUT /` Listing or creating feeds, not the content
  - `GET/UPDATE/DELETE /:id`

- Index `/index`
  - ID can be derived from the parameters and then hashed for caching
  - Two types: One shows the content as-is, the other builds it from the RSS history
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
      - `read-later`, `history`, ... as custom IDs, create if doesn't exist
      - `GET /list` Gets the items in the list
      - `PUT/DELETE /:item_id`

- User `/user`
  - `POST /login`
  - `PUT /register` Register
  - `/:id`
    - `GET/UPDATE /settings`
    - `POST /reset-password`

- Search `/search`
  - `GET /:query` How does this work? Postgres full text search. Use feedbin for inspiration

## Roadmap

- [x] API Schema design
- [ ] Axum template with SQLx
- [ ] Postgres schema
- [ ] RSS parsing and caching
- [ ] Website parsing
- [ ] API Implementation
  - [ ] Feeds
  - [ ] Index
  - [ ] Items
    - [ ] Lists
  - [ ] Users
    - [ ] Authentication
  - [ ] Search
