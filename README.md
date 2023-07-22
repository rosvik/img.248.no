# img.248.no

A simple Rust-based tool and API that allows you to resize images from different formats (JPEG, PNG, GIF) hosted at a given URL. The API uses the Axum framework for handling HTTP requests and responses and the Serde library for deserializing query parameters.

Can be set up as a image optimizer for your front end, for example as a [Next.js image loader](https://nextjs.org/docs/pages/api-reference/next-config-js/images).

## How to Use

### Setup

```bash
cargo run
```

The API will start running on `http://127.0.0.1:2338`.

### Usage

`GET /<FILENAME>?url=<URL>&w=<WIDTH>&h=<HEIGHT>`

Resizes the image specified by the `URL` to the provided `WIDTH` and `HEIGHT` parameters. `FILENAME` should have the file extension `.jpg`, `.png`, or `.gif`.

Parameters:
- `url`: The URL of the image to resize.
- `w`: Optional. The desired width of the resized image.
- `h`: Optional. The desired height of the resized image.

_Providing only one of `w` or `h` will give you an image with preserved aspect ratio. When providing neither, the image will not be resized._

Example:
```bash
curl "http://127.0.0.1:2338/image-name.jpg?url=https://example.com/image.png&w=100"
```
