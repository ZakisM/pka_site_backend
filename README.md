![Build and Test](https://github.com/ZakisM/pka_site_backend/workflows/Build%20and%20Test/badge.svg)

# PKA Index Backend

This repo holds the backend code for https://www.pkaindex.com.

To run locally:

#### To develop/test

1. Download rust: https://rustup.rs/.
2. (Optional) Install Diesel CLI for making changes to DB (You will need to install sqlite3 even if you skip this
   step): https://github.com/diesel-rs/diesel/tree/master/diesel_cli
3. Download and install `redis`. Make sure `redis-server` is running on its default port `6379`.
4. Modify your hosts file and add the following entries if they aren't there already:
    - 127.0.0.1 redis
    - 0.0.0.0 pkaindextest.com
5. Create a folder somewhere i.e `/Users/zak/Desktop/selfsigned/` and generate self-signed SSL Certificate in this
   folder:
    - NOTE: When openssl asks you to enter `Common name:` put `pkaindextest.com`
    - `openssl req -new -x509 -days 365 -nodes -out /Users/zak/Desktop/selfsigned/pkaindextest.pem -keyout /Users/zak/Desktop/selfsigned/pkaindextest.key -newkey rsa:2048`
6. Modify `nginx.conf` from this project and replace the lines that have `YOUR_SELF_SIGNED_KEY_DIRECTORY` with the
   folder you created in the previous step. i.e `/Users/zak/Desktop/selfsigned/`
7. Install `nginx` and modify `/etc/nginx/nginx.conf` on your machine, so it is the same as `nginx.conf` that you just
   modified from this project.
8. Start `nginx`.
9. Generate Youtube API key from https://console.developers.google.com/ and save as env variable named: YT_API_KEY. Can
   pass empty string if you want however this means episodes won't be updated.
10. Run the rust server
    - To run in debug mode: run `cargo run` from project root.
    - To run in release (optimized) mode: run `cargo run --release` from project root.
11. Rust should now be serving an API from http://0.0.0.0:1234.
12. Visit https://pkaindextest.com in your browser. (Firefox will work but for Chrome you will need to import the Self
    Signed SSL certificate manually.)

#### Test With Docker - Note this is creating an optimized build so not suitable for development.

1. `docker build -t zakism/pka-index-backend:latest .`
2. `docker run -p 1234:1234 zakism/pka-index-backend:latest`
3. Rust should now be serving an API from http://0.0.0.0:1234.

### Alternative way to test backend and frontend with docker:

1. `docker-compose up -d`

