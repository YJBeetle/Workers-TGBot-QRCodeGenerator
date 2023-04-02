use worker::*;

use image::{DynamicImage, Rgb};
use qrcode::QrCode;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get("/", |_, _| Response::ok("Hello"))
        .get("/generator", |req, _| {
            let url = req.url()?;
            let data = url
                .query_pairs()
                .find(|(key, _)| key == "data")
                .map(|(_, value)| value.to_string())
                .unwrap_or_default();

            // 生成二维码图像
            let qr_code = QrCode::new(data.as_bytes()).unwrap();
            let image = qr_code.render::<Rgb<u8>>().build();

            // 将图像转换为 PNG 格式
            let png_image = DynamicImage::ImageRgb8(image);
            let mut png_data = Vec::new();
            png_image
                .write_to(&mut png_data, image::ImageOutputFormat::Png)
                .unwrap();

            let mut headers = Headers::new();
            headers.set("content-type", "image/png")?;

            Ok(Response::from_bytes(png_data)?.with_headers(headers))
        })
        .run(req, env)
        .await
}
