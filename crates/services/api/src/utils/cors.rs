use kestrel_config::structs::server::CorsConfig;
use rocket::{
    Request, Response,
    fairing::{Fairing, Info, Kind},
    http::{Header, Method, Status},
    response::status::NoContent,
};

#[options("/<_..>")]
pub fn preflight() -> NoContent {
    NoContent
}

pub struct CorsFairing {
    pub config: CorsConfig,
}

#[rocket::async_trait]
impl Fairing for CorsFairing {
    fn info(&self) -> Info {
        Info {
            name: "Kestrel REST API CORS Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let origin = req.headers().get_one("Origin");

        res.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        ));

        res.set_header(Header::new(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        ));

        if req.method() == Method::Options {
            res.set_header(Header::new("Access-Control-Max-Age", "86400"));
            res.set_status(Status::NoContent);
        }

        let allow_all = self.config.allowed_origins.contains(&"*".into());

        if self.config.allow_credentials {
            if let Some(origin) = origin
                && (allow_all || self.config.allowed_origins.contains(&origin.into()))
            {
                res.set_header(Header::new("Access-Control-Allow-Origin", origin));
                res.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
                res.set_header(Header::new("Vary", "Origin"));
            }
        } else {
            if allow_all {
                res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
            } else if let Some(origin) = origin
                && self.config.allowed_origins.contains(&origin.into())
            {
                res.set_header(Header::new("Access-Control-Allow-Origin", origin));
            }
        }
    }
}
