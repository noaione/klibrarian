# 📚 K-Librarian

A simple web server to create an invite system for Komga and Navidrome.<br />

Powered by [Axum](https://github.com/tokio-rs/axum) and SQLite3 for high performance and memory efficient web server.

## Requirements
1. Node.js 18.x or higher
2. Rust 1.88.0 or higher
3. Komga server
4. Navidrome server (optional, if you want to use Navidrome)

## Installing
Download new releases at: https://github.com/noaione/klibrarian/releases

Or, get the latest development build at: https://github.com/noaione/klibrarian/actions/workflows/ci.yml<br />
Just click on the latest successful build at download the `k-librarian` artifacts

## Compiling
1. Install Node/NPM, Rust and Redis
2. Clone this repository
3. [Configure](#configuration) your instances
4. Install dependencies for frontend: `npm install`
5. Build the frontend using: `npm run build`
6. Run cargo build: `cargo build --profile production --locked`
   - If your assets folder is empty and nothing is copying, manually copy the `frontend/dist` folder contents into `assets/`, do not include the `index.html`
7. Execute the target file in `target/release/k-librarian`
8. Open: http://127.0.0.1:5148

## Configuration
The invite system is configured using a `config.toml` file, you can copy the example file from `config.toml.example` to `config.toml` and edit it.

```toml
# The configuration file for k-librarian

# This is the host/port where the k-librarian web server will run.
host = "127.0.0.1"
port = 5148

# Your auth token, to access the admin panel.
token = "this-is-your-auth-token"

# Database path, relative to the current working directory.
# or the absolute path to the database file.
db-path = "./.klibrarian/database.sqlite"

[komga]
# Host and port of the Komga instance
host = "https://demo.komga.org"
# Username and password for the Komga instance
username = "demo@komga.org"
password = "demo"
# The actual hostname of Komga, if you prefer to put host as localhost and you're running
# behind a reverse proxy, you can define this for the actual instances URL.
# hostname = "https://demo.komga.org"

# [navidrome]
# # Host and port of the Navidrome instance
# host = "https://demo.navidrome.org"
# # Username and password for the Navidrome instance
# username = ""
# password = ""
# # The actual hostname of Navidrome, if you prefer to put host as localhost and you're running
# # behind a reverse proxy, you can define this for the actual instances URL.
# # hostname = "https://demo.navidrome.org"
```

## Attribution

The icon/favicon/logo used by K-Librarian is a non-modified version of icon called **books icon** by Freepik: [Flaticon](https://www.flaticon.com/free-icon/books_3771417?term=books&page=1&position=14&origin=tag&related_id=3771417)<br />
All rights reserved to the original creator.

Komga icon and Navidrome icon are used as is, without modifications, and all rights reserved to the original creators.
