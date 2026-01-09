use reqwest::Client;

#[derive(Clone)]
pub struct RemoteWriter {
    vm_url: String,
    client: Client,
}

impl RemoteWriter {
    pub fn new(vm_url: String, client: Client) -> Self {
        RemoteWriter { vm_url, client }
    }
    pub async fn send(&self, text: String) -> Result<()> {
        let res = self
            .client
            .post(self.vm_url.clone())
            .body(text)
            .header("Content-Type", "text/plain")
            .send()
            .await?;
        res.error_for_status()?;
        Ok(())
    }
}
