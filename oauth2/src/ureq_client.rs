use crate::{HttpClientError, HttpRequest, HttpResponse};

use http::{header::CONTENT_TYPE, method::Method};

impl crate::SyncHttpClient for ureq::Agent {
    type Error = HttpClientError<ureq::Error>;

    fn call(&self, request: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let uri = request.uri().to_string();
        let headers = request.headers();
        let response = if *request.method() == Method::POST {
            let mut req = self.post(&uri);
            for (name, value) in headers {
                req = req.header(
                    name.as_str(),
                    value.to_str().map_err(|_| {
                        HttpClientError::Other(format!(
                            "invalid `{name}` header value {:?}",
                            value.as_bytes()
                        ))
                    })?,
                );
            }
            req.send(request.body())
        } else {
            debug_assert_eq!(*request.method(), Method::GET);
            let mut req = self.get(&uri);
            for (name, value) in headers {
                req = req.header(
                    name.as_str(),
                    value.to_str().map_err(|_| {
                        HttpClientError::Other(format!(
                            "invalid `{name}` header value {:?}",
                            value.as_bytes()
                        ))
                    })?,
                );
            }
            req.call()
        }
        .map_err(Box::new)?;

        let mut builder = http::Response::builder().status(response.status());

        if let Some(content_type) = response.headers().get(CONTENT_TYPE.as_str()) {
            builder = builder.header(CONTENT_TYPE, content_type);
        }

        let body = response.into_body().read_to_vec().map_err(Box::new)?;
        builder.body(body).map_err(HttpClientError::Http)
    }
}
