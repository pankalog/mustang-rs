# Mustang

A simple, blazing-fast URL shortener made in Rust using Redis and Axum.

## Description

Mustang is a URL shortener service that leverages the speed and efficiency of Redis and the Axum web framework. It provides a straightforward API for creating and retrieving shortened URLs.

## Installation

To install and run Mustang, you need to have Rust and Cargo installed. Clone the repository and build the project using Cargo:

```sh
git clone https://github.com/pankalog/mustang-rs
cd mustang-rs
cargo build --release
```

## Usage

Before running the application, ensure you have a Redis server running and set the necessary environment variables:

```sh
export REDIS_CONN_STRING=redis://127.0.0.1:6379/
export BIND_HOST=127.0.0.1
export BIND_PORT=8080
```

Run the application:

```sh
cargo run --release
```

## API Endpoints

### Create a Shortened URL

**`POST /`**

Request Body:
```json
{
  "destination_url": "https://example.com"
}
```

Response:
```json
{
  "short_id": "abc123",
  "full_url": "http://localhost:8080/abc123",
  "destination_url": "https://example.com"
}
```

### Retrieve a Shortened URL

**`GET /{shortened_id}`**

Response:
- **307 Temporary Redirect** to the destination URL if found.
- **404 Not Found** if the shortened ID does not exist.

### OpenAPI Documentation

**`GET /openapi.json`**

Returns the OpenAPI documentation for the API in JSON format. You can also find it on the root of the repository.

## Authors

- Panos Kalogeropoulos - [pkal.dev](https://pkal.dev)