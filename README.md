![Build and Test](https://github.com/ZakisM/pka_site_backend/workflows/Build%20and%20Test/badge.svg)

# PKA Index Backend

This repo holds the backend code for https://www.pkaindex.com.

To run locally:

#### To develop/test

1. Download rust: https://rustup.rs/.
2. (Optional) Install `sqlx-cli` for running migrations and refreshing query metadata (You will need
   `sqlite3` even if you skip this step): https://github.com/launchbadge/sqlx/tree/main/sqlx-cli
   - Apply migrations (uses `DATABASE_URL` from `.env`): `sqlx migrate run`
   - After altering SQL or migrations run `cargo sqlx prepare -- --bin pka_site_backend` so the
     checked-in `.sqlx/` data stays in sync.
3. Modify your hosts file and add the following entries if they aren't there already:
    - 0.0.0.0 pkaindextest.com www.pkaindextest.com
4. Create a folder somewhere i.e `/Users/zak/Selfsigned/` and generate self-signed SSL Certificate in this
   folder:
    - NOTE: When openssl asks you to enter `Common name:` put `pkaindextest.com`
    - `openssl req -new -x509 -days 365 -nodes -out /Users/zak/Selfsigned/pkaindextest.pem -keyout /Users/zak/Selfsigned/pkaindextest.key -newkey rsa:2048`
5. Modify `nginx.conf` from this project and replace the lines that have `YOUR_SELF_SIGNED_KEY_DIRECTORY` with the
   folder you created in the previous step. i.e `/Users/zak/Selfsigned/`
6. Install `nginx` and create the directories `sites-enabled` and `sites-available` in your `/etc/nginx` directory.
7. Copy the contents from `nginx.conf` into a file called `pkaindextest.com` inside `sites-availabled`
8. Create a symbolic link between the file in `sites-available` and `sites-enabled` using the
   command: `ln -s /opt/homebrew/etc/nginx/sites-available/pkaindextest.com /opt/homebrew/etc/nginx/sites-enabled/pkaindextest.com`
9. Modify `nginx.conf` in your system so that your `http` block includes this line: `include sites-available/*;`;
10. Start `nginx`.
11. Generate YouTube API key from https://console.developers.google.com/ and save as env variable named: YT_API_KEY. Can
    pass empty string if you want however this means episodes won't be updated.
12. Run the rust server
    - To run in debug mode: run `cargo run` from project root.
    - To run in release (optimized) mode: run `cargo run --release` from project root.
13. Rust should now be serving an API from http://0.0.0.0:1234.
    - OpenAPI schema is available at http://0.0.0.0:1234/openapi.json.
14. Visit https://pkaindextest.com in your browser. (Firefox will work but for Chrome you will need to import the Self
    Signed SSL certificate manually.)

#### Test With Docker - Note this is creating an optimized build so not suitable for development.

1. `docker build -t zakism/pka-index-backend:latest .`
2. `docker run -p 1234:1234 zakism/pka-index-backend:latest`
3. Rust should now be serving an API from http://0.0.0.0:1234.

### Alternative way to test backend and frontend with docker:

1. `docker-compose up -d`
