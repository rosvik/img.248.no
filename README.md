# img.248.no

A simple Rust-based tool and API that allows you to resize images from different formats (JPEG, PNG, GIF) hosted at a given URL. The API uses the Axum framework for handling HTTP requests and responses and the Serde library for deserializing query parameters.

You're welcome to use the hosted version at img.248.no for testing, but I can't guarantee that it will be reliable or able to handle a significant amount of traffic.

## How to Use

### Setup

```bash
cargo run
```

The API will start running on `http://127.0.0.1:2338`.

### Resizing an image

`GET /<FILENAME>?url=<URL>&w=<WIDTH>&h=<HEIGHT>`

Resizes the image specified by the `URL` to the provided `WIDTH` and `HEIGHT` parameters. `FILENAME` should have the file extension `.jpg`, `.png`, or `.gif`.

### Parameters

- `url`: Required. The URL of the image to resize.
- `w`: The desired width of the resized image. Default is the original width, or auto if height is provided.
- `h`: The desired height of the resized image. Default is the original height, or auto if width is provided.
- `mode`: The resizing mode to use when both width and height is provided, and aspect ratio is different from the source image. Accepts one of `crop`, `fit`, or `stretch`. Default is `crop`.
- `quality`: The quality of the resized image from 0 to 100. Only applies to JPEG images. Default is 100.
- `sampling`: The sampling filter to use when resizing the image. Accepts one of `nearest`, `linear`, `cubic`, `gaussian`, `lanczos`, or `best`. Default is `linear`. For examples and info on performance, see [image::imageops::FilterType](https://docs.rs/image/latest/image/imageops/enum.FilterType.html).
- `base64`: If set, the image will be returned as a base64-encoded string. (Accepts `true`/`false` or `on`/`off` as values, or simply `&base64`)

### Example

The url https://img.248.no/example.jpg?url=https://picsum.photos/seed/h/1000/700&w=800&quality=90 gives this 800px wide JPEG image with 90% quality:

<div align="center">
  <a href="https://img.248.no/example.jpg?url=https://picsum.photos/seed/h/1000/700&w=800&quality=90">
    <img src="https://img.248.no/example.jpg?url=https://picsum.photos/seed/h/1000/700&w=800&quality=90">
  </a>
</div>

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
  return `https://img.248.no/image.jpg?&url=${src}&w=${width}`;
}
```

<div align="right"><img src="https://github-production-user-asset-6210df.s3.amazonaws.com/1774972/269361517-d0d8e30e-4a25-4ba2-b926-2a42da1156f8.svg" width="32" alt="248"></div>
