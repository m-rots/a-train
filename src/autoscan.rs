use bernard::{ChangedPath, Path};
use reqwest::{Client, ClientBuilder, IntoUrl, Url};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::{collections::HashSet, path::PathBuf};
use tracing::debug;

pub(crate) struct Autoscan {
    auth: Option<Credentials>,
    client: Client,
    url: Url,
}

impl Autoscan {
    pub(crate) fn new(auth: Option<Credentials>, client: Client, url: Url) -> Self {
        Self { auth, client, url }
    }

    pub(crate) fn builder<U: IntoUrl>(url: U, auth: Option<Credentials>) -> AutoscanBuilder {
        AutoscanBuilder::new(url, auth)
    }
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

pub(crate) struct AutoscanBuilder {
    auth: Option<Credentials>,
    client: ClientBuilder,
    url: Url,
}

impl AutoscanBuilder {
    pub(crate) fn new<U: IntoUrl>(url: U, auth: Option<Credentials>) -> Self {
        let url = url.into_url().expect("Invalid url");

        AutoscanBuilder {
            auth,
            client: ClientBuilder::new(),
            url,
        }
    }

    pub(crate) fn proxy<U: IntoUrl>(mut self, url: U) -> Self {
        let proxy = reqwest::Proxy::all(url).unwrap();

        self.client = self.client.proxy(proxy);
        self
    }

    pub(crate) fn build(self) -> Autoscan {
        let client = self.client.build().unwrap();
        Autoscan::new(self.auth, client, self.url)
    }
}

#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize, PartialEq))]
pub(crate) struct Payload {
    created: HashSet<PathBuf>,
    deleted: HashSet<PathBuf>,
}

impl Payload {
    pub(crate) fn is_empty(&self) -> bool {
        self.created.len() == 0 && self.deleted.len() == 0
    }
}

pub(crate) fn create_payload(changed_paths: Vec<ChangedPath>) -> Payload {
    let mut payload = Payload::default();

    for path in changed_paths {
        match path {
            ChangedPath::Created(path) => match path {
                Path::File(mut file) => {
                    // We're only interested in folders.
                    // Thus we pop the file and retrieve the parent instead.
                    file.path.pop();
                    payload.created.insert(file.path);
                }
                Path::Folder(folder) => {
                    payload.created.insert(folder.path);
                }
            },
            ChangedPath::Deleted(path) => match path {
                Path::File(mut file) => {
                    // We're only interested in folders.
                    // Thus we pop the file and retrieve the parent instead.
                    file.path.pop();
                    payload.deleted.insert(file.path);
                }
                Path::Folder(folder) => {
                    payload.deleted.insert(folder.path);
                }
            },
        }
    }

    payload
}

impl Autoscan {
    #[tracing::instrument(skip(self, payload))]
    pub(crate) async fn send_payload(
        &self,
        drive_id: &str,
        payload: &Payload,
    ) -> crate::Result<()> {
        let mut url = self.url.clone();
        url.set_path(&format!("/triggers/a-train/{}", drive_id));

        let mut request = self.client.post(url).json(&payload);
        if let Some(auth) = &self.auth {
            request = request.basic_auth(&auth.username, Some(&auth.password));
        }

        request
            .send()
            .await
            .context(crate::AutoscanUnavailable)?
            .error_for_status()
            .context(crate::AutoscanUnavailable)?;

        debug!("changes received by autoscan");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{create_payload, Autoscan, Payload};
    use bernard::{ChangedPath, InnerPath, Path};
    use pretty_assertions::assert_eq;
    use reqwest::{Client, Url};
    use serde_json::{from_value, json};
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, ResponseTemplate};

    fn new_path(created: bool, folder: bool, inner: InnerPath) -> ChangedPath {
        match (created, folder) {
            (true, true) => ChangedPath::Created(Path::Folder(inner)),
            (false, true) => ChangedPath::Deleted(Path::Folder(inner)),
            (true, false) => ChangedPath::Created(Path::File(inner)),
            (false, false) => ChangedPath::Deleted(Path::File(inner)),
        }
    }

    fn new_inner(path: &str) -> InnerPath {
        InnerPath {
            // drive_id and id are not used, so whatever
            drive_id: "test".to_string(),
            id: "test".to_string(),
            path: path.into(),
        }
    }

    impl Autoscan {
        fn new_test(url: &str) -> Self {
            Self {
                auth: None,
                client: Client::new(),
                url: Url::parse(&url).unwrap(),
            }
        }
    }

    #[tokio::test]
    async fn autoscan_request() {
        let server = wiremock::MockServer::start().await;
        let autoscan = Autoscan::new_test(&server.uri());

        let payload: Payload = create_payload(vec![
            new_path(true, true, new_inner("/this/is/a/full/path")),
            new_path(false, true, new_inner("/just/like/me")),
        ]);

        let expected_body = json!({
            "created": [
                "/this/is/a/full/path",
            ],
            "deleted": [
                "/just/like/me"
            ],
        });

        Mock::given(method("POST"))
            .and(path("/triggers/a-train/test123"))
            .and(body_json(&expected_body))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let result = autoscan.send_payload("test123", &payload).await;

        // First drop the server to evaluate the request.
        drop(server);
        // Afterwards, check the result.
        // This should happen last as the panic information is pretty useless.
        result.unwrap();
    }

    /// Check whether folder paths keep as is.
    #[test]
    fn payload_folders_are_full_paths() {
        let payload: Payload = create_payload(vec![
            new_path(true, true, new_inner("/this/is/a/full/path")),
            new_path(false, true, new_inner("/just/like/me")),
        ]);

        let expected_body = json!({
            "created": [
                "/this/is/a/full/path",
            ],
            "deleted": [
                "/just/like/me"
            ],
        });

        assert_eq!(
            payload,
            from_value(expected_body).expect("could not deserialize")
        )
    }

    /// Check whether file paths are transformed into the path of the parent.
    #[test]
    fn payload_files_are_parents() {
        let payload: Payload = create_payload(vec![
            new_path(true, false, new_inner("/keep me/but not me")),
            new_path(false, false, new_inner("/where/is/perry")),
        ]);

        let expected_body = json!({
            "created": [
                "/keep me",
            ],
            "deleted": [
                "/where/is"
            ],
        });

        assert_eq!(
            payload,
            from_value(expected_body).expect("could not deserialize")
        )
    }
}
