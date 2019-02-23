use std::io::Cursor;

use rocket::http::{Status, ContentType};
use rocket_contrib::json::JsonValue;
use rocket::request::Request;
use rocket::response::Responder;
use rocket::response::Response as OriginalResponse;

#[derive(Debug)]
pub struct Response {
    pub status: Status,
    pub data: JsonValue,
}

impl Response {
    pub fn data(mut self, data: JsonValue) -> Response {
        self.data = data;
        self
    }
}

impl<'r> Responder<'r> for Response {
    fn respond_to(
        self,
        _req: &Request,
    ) -> Result<OriginalResponse<'r>, Status>
    {
        let body = self.data;

        OriginalResponse::build()
            .status(self.status)
            .sized_body(Cursor::new(body.to_string()))
            .header(ContentType::JSON)
            .ok()
    }
}
