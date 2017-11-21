/// A file uploaded with a message. Not to be confused with [`Embed`]s.
///
/// [`Embed`]: struct.Embed.html
#[derive(Clone, Debug, Deserialize)]
pub struct Attachment {
    /// The unique ID given to this attachment.
    pub id: String,
    /// The filename of the file that was uploaded. This is equivalent to what
    /// the uploader had their file named.
    pub filename: String,
    /// If the attachment is an image, then the height of the image is provided.
    pub height: Option<u64>,
    /// The proxy URL.
    pub proxy_url: String,
    /// The size of the file in bytes.
    pub size: u64,
    /// The URL of the uploaded attachment.
    pub url: String,
    /// If the attachment is an image, then the width of the image is provided.
    pub width: Option<u64>,
}
