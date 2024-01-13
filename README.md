# img.248.no

A simple Rust-based tool and API that allows you to resize images from different formats (JPEG, PNG, GIF) hosted at a given URL. The API uses the Axum framework for handling HTTP requests and responses and the Serde library for deserializing query parameters.

You're welcome to use the hosted version at img.248.no for testing, but you should host your own instance if you care about reliability, or expect a significant amount of traffic.

## How to Use

### Setup

```bash
cargo run
```

The API will start running on `http://127.0.0.1:2338`.

### Resizing an image

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

### Next.js image loader

This tool is tailored to be set up as an image optimizer for front ends, like [Next.js image loaders](https://nextjs.org/docs/app/api-reference/next-config-js/images).

In order to configure Next.js to use this tool, you need to add the following to your `next.config.js` file:

```js
const config = {
  images: {
    loader: "custom",
    loaderFile: "./path/to/image-loader.js",
  },
};
```

Then, create `image-loader.js` with the following contents:

```js
export default function imageLoader({ src, width }) {
  return `https://img.248.no/image.jpg?&w=${width}&url=${src}`;
}
```

<div align="right"><img src="https://github-production-user-asset-6210df.s3.amazonaws.com/1774972/269361517-d0d8e30e-4a25-4ba2-b926-2a42da1156f8.svg" width="32" alt="248"></div>
