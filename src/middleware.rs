use rocket::data::{FromDataSimple, Outcome};
use rocket::http::Status;
use rocket::{Data, Outcome::*, Request};
use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions, RawField, TextField,
};
use serde::{Deserialize, Serialize};

// first we need to create a custom error type, as the FromDataSimple guard
// needs to return one
#[derive(Debug, Clone)]
pub struct MultipartError {
    pub reason: String,
}

impl MultipartError {
    fn new(reason: String) -> MultipartError {
        MultipartError { reason }
    }
}

impl std::fmt::Display for MultipartError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

/// simple representation of a user
#[derive(Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub age: i32,
}

pub struct NewUser {
    /// the submitted image
    pub avatar: Vec<u8>,
    /// we'll deserialize the json into a User
    pub user: User,
}
impl FromDataSimple for NewUser {
    type Error = MultipartError;

    fn from_data(request: &Request, data: Data) -> Outcome<Self, Self::Error> {
        let image_bytes;
        let post_obj;
        let mut options = MultipartFormDataOptions::new();

        // setup the multipart parser, this creates a parser
        // that checks for two fields: an image of any mime type
        // and a data field containining json representing a User
        options.allowed_fields.push(
            MultipartFormDataField::raw("avatar")
                .size_limit(8 * 1024 * 1024) // 8 MB
                .content_type_by_string(Some(mime::IMAGE_STAR))
                .unwrap(),
        );
        options
            .allowed_fields
            .push(MultipartFormDataField::text("data").content_type(Some(mime::STAR_STAR)));

        // check if the content type is set properly
        let ct = match request.content_type() {
            Some(ct) => ct,
            _ => {
                return Failure((
                    Status::BadRequest,
                    MultipartError::new(format!(
                        "Incorrect contentType, should be 'multipart/form-data"
                    )),
                ))
            }
        };

        // do the form parsing and return on error
        let multipart_form = match MultipartFormData::parse(&ct, data, options) {
            Ok(m) => m,
            Err(e) => {
                return Failure((Status::BadRequest, MultipartError::new(format!("{:?}", e))))
            }
        };
        // check if the form has the json field `data`
        let post_json_part = match multipart_form.texts.get("data") {
            Some(post_json_part) => post_json_part,
            _ => {
                return Failure((
                    Status::BadRequest,
                    MultipartError::new(format!("Missing field 'data'")),
                ))
            }
        };
        // check if the form has the avatar image
        let image_part: &RawField = match multipart_form.raw.get("avatar") {
            Some(image_part) => image_part,
            _ => {
                return Failure((
                    Status::BadRequest,
                    MultipartError::new(format!("Missing field 'avatar'")),
                ))
            }
        };
        // verify only the data we want is being passed, one text field and one binary
        match post_json_part {
            TextField::Single(text) => {
                let json_string = &text.text.replace('\'', "\"");
                post_obj = match serde_json::from_str::<User>(json_string) {
                    Ok(insert) => insert,
                    Err(e) => {
                        return Failure((
                            Status::BadRequest,
                            MultipartError::new(format!("{:?}", e)),
                        ))
                    }
                };
            }
            TextField::Multiple(_text) => {
                return Failure((
                    Status::BadRequest,
                    MultipartError::new(format!("Extra text fields supplied")),
                ))
            }
        };
        match image_part {
            RawField::Single(raw) => {
                image_bytes = raw.raw.clone();
            }
            RawField::Multiple(_raw) => {
                return Failure((
                    Status::BadRequest,
                    MultipartError::new(format!("Extra image fields supplied")),
                ))
            }
        };
        Success(NewUser {
            user: post_obj,
            avatar: image_bytes,
        })
    }
}
