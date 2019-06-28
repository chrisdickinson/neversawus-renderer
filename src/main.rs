#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate simple_logger;

use lambda::error::HandlerError;
use aws_lambda_events::event::s3::S3Event;
use std::error::Error;
use comrak::{ markdown_to_html, ComrakOptions };

use s3::bucket::Bucket;
use s3::region::Region;
use s3::credentials::Credentials;

#[derive(Serialize, Clone)]
struct CustomOutput {
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info)?;
    lambda!(my_handler);
    Ok(())
}

struct Post {
    title: String,
    body: String
}

fn fetch_markdown(bucket: &str, key: &str) -> Result<Vec<u8>, std::io::Error> {
    let creds = Credentials::new(
        std::env::var("S3_SOURCE_ACCESS_KEY").ok(),
        std::env::var("S3_SOURCE_SECRET_KEY").ok(),
        None,
        None
    );

    let region = std::env::var("S3_SOURCE_REGION").ok().unwrap().parse().unwrap_or_else(|_| Region::UsWest2);
    let bucket = Bucket::new(bucket, region, creds).unwrap();
    let (data, code) = bucket.get_object(key).unwrap();
    if code == 200 {
        return Ok(data)
    }

    return Err(std::io::ErrorKind::NotFound.into());
}

fn render_markdown(markdown: Vec<u8>) -> Result<Post, std::io::Error> {
    let as_string = std::str::from_utf8(&markdown).ok().unwrap_or_else(|| "failed to convert to utf8");
    let mut iter = as_string.splitn(1, "\n---\n\n");

    let title = iter.next().unwrap().to_string();
    let body = markdown_to_html(iter.next().unwrap(), &ComrakOptions::default());

    return Ok(Post {
        title,
        body
    })
}

fn upload_to_s3(key: &str, result: Post) -> Result<(), std::io::Error> {
    Ok(())
}

fn my_handler(e: S3Event, c: lambda::Context) -> Result<CustomOutput, HandlerError> {
    for record in e.records {
        if record.event_name.is_none() {
            continue
        }

        let event_name = record.event_name.unwrap();
        match event_name.as_ref() {
            "ObjectCreated:Put" => {
                let bucket = record.s3.bucket.name.unwrap_or_else(|| String::from(""));
                let key = record.s3.object.key.unwrap_or_else(|| String::from(""));

                if key.len() == 0 {
                    return Ok(CustomOutput { message: "skipped (key not given)".to_string() })
                }
                if bucket.len() == 0 {
                    return Ok(CustomOutput { message: "skipped (bucketnot given)".to_string() })
                }

                if !key.ends_with(".md") {
                    return Ok(CustomOutput { message: "skipped (not markdown)".to_string() })
                }

                let markdown = match fetch_markdown(&bucket, &key) {
                    Ok(xs) => xs,
                    Err(e) => {
                        return Ok(CustomOutput { message: format!("failed to fetch {}/{}", bucket, key) })
                    }
                };

                let result = match render_markdown(markdown) {
                    Ok(xs) => xs,
                    Err(e) => {
                        return Ok(CustomOutput { message: e.description().to_string() })
                    }
                };

                match upload_to_s3(&key, result) {
                    Ok(xs) => xs,
                    Err(e) => {
                        return Ok(CustomOutput { message: format!("failed to fetch {}/{}", bucket, key) })
                    }
                };
            }
            _ => {
                error!("unrecognized event type {}", event_name);
                return Ok(CustomOutput { message: "skipped".to_string() });
            }
        };
    }

    Ok(CustomOutput {
        message: format!("Hello!"),
    })
}
