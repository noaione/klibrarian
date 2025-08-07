# 📚 K-Librarian

A simple web server to create an invite system for Komga.<br />

Made on top of [Axum](https://github.com/tokio-rs/axum) for high performance and memory efficient web server.

## Requirements
1. Node.js 18.x or higher
2. Rust 1.88.0 or higher
3. Redis-compatible server (any pre-SSPL fork, Garnet, etc.)
4. Komga server

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
6. Run cargo build: `cargo build --release`
   - If your assets folder is empty and nothing is copying, manually copy the `frontend/dist` folder contents into `assets/`, do not include the `index.html`
7. Execute the target file in `target/release/k-librarian`
8. Open: http://127.0.0.1:5148

## Configuration
The invite system use simple dotenv (`.env`) file to configure the server, you can see the `.env.example`
file, copy into `.env` and modify it.

```conf
# The host and port of the server
HOST=127.0.0.1
PORT=5148

# Set the token for authorization in the web ui, make sure it's secure!
# TOKEN=

### Komga configuration
# The host of the komga server
KOMGA_HOST=https://demo.komga.org
# The username of the komga server
KOMGA_USERNAME=demo@komga.org
# The password of the komga server
KOMGA_PASSWORD=demo
# The actual hostname of Komga, if you prefer to put KOMGA_HOST as localhost and you're running behind reverse
# proxy, define this for the actual instances URL.
# KOMGA_HOSTNAME=

### Redis configuration
# Host and port of the redis server
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
# Password of the redis server, uncomment if needed
# REDIS_PASS=
```

## Attribution

The icon/favicon/logo used by K-Librarian is a non-modified version of icon called **books icon** by Freepik: [Flaticon](https://www.flaticon.com/free-icon/books_3771417?term=books&page=1&position=14&origin=tag&related_id=3771417)<br />
All rights reserved to the original creator.
